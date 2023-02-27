use anyhow::Result;
use nalgebra_glm as glm;
use phantom_world::{Vertex, World};
use std::{
    borrow::Cow,
    mem::{self, size_of},
};
use wgpu::{
    self,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, Buffer, BufferAddress, Device, Face, Queue, RenderPass, RenderPipeline,
    TextureFormat, VertexAttribute,
};

pub struct WorldRender {
    pub geometry: Geometry,
    pub uniform: UniformBinding,
    pub dynamic_uniform: DynamicUniformBinding,
    pub pipeline: RenderPipeline,
}

impl WorldRender {
    pub fn new(device: &Device, surface_format: TextureFormat, world: &World) -> Self {
        let geometry = Geometry::new(device, &world.geometry.vertices, &world.geometry.indices);
        let uniform = UniformBinding::new(device);
        let dynamic_uniform = DynamicUniformBinding::new(device);
        let pipeline = create_pipeline(device, surface_format, &uniform, &dynamic_uniform);
        Self {
            geometry,
            uniform,
            dynamic_uniform,
            pipeline,
        }
    }

    pub fn render<'rp>(&'rp self, render_pass: &mut RenderPass<'rp>, world: &World) -> Result<()> {
        let metadata = world.get_metadata();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform.bind_group, &[]);

        let (vertex_buffer_slice, index_buffer_slice) = self.geometry.slices();
        render_pass.set_vertex_buffer(0, vertex_buffer_slice);
        render_pass.set_index_buffer(index_buffer_slice, wgpu::IndexFormat::Uint32);

        for entity_metadata in metadata.iter() {
            let offset = (entity_metadata.offset as wgpu::DynamicOffset)
                * self.dynamic_uniform.alignment as wgpu::DynamicOffset;
            render_pass.set_bind_group(1, &self.dynamic_uniform.bind_group, &[offset]);
            render_pass.draw_indexed(entity_metadata.index_range.clone(), 0, 0..1);
        }

        Ok(())
    }

    pub fn update(&mut self, queue: &Queue, aspect_ratio: f32, world: &World) {
        let (projection, view) = world.active_camera_matrices(aspect_ratio).unwrap();
        let camera_entity = world.active_camera().unwrap();
        let camera_transform = world.entity_global_transform(camera_entity).unwrap();
        let camera_position = glm::vec3_to_vec4(&camera_transform.translation);

        let lights = world.components::<phantom_world::Light>().unwrap();
        let (transform, light) = lights.first().unwrap();
        let light = Light::new(transform.translation, light.color);

        self.uniform.upload_uniform_data(
            queue,
            0,
            Uniform {
                view,
                projection,
                camera_position,
                light,
            },
        );

        let mut mesh_ubos =
            vec![DynamicUniform::default(); DynamicUniformBinding::MAX_NUMBER_OF_MESHES];
        let mut ubo_offset = 0;
        for graph in world.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    let model = world.global_transform(graph, node_index)?;
                    mesh_ubos[ubo_offset] = DynamicUniform { model };
                    ubo_offset += 1;
                    Ok(())
                })
                .unwrap();
        }
        self.dynamic_uniform
            .upload_uniform_data(queue, 0, &mesh_ubos);
    }
}

fn create_pipeline(
    device: &Device,
    surface_format: TextureFormat,
    uniform: &UniformBinding,
    dynamic_uniform: &DynamicUniformBinding,
) -> RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER_SOURCE)),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &uniform.bind_group_layout,
            &dynamic_uniform.bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[create_vertex_description(&create_vertex_attributes())],
        },
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    })
}

pub fn create_vertex_attributes() -> Vec<VertexAttribute> {
    vertex_attr_array![
    0 => Float32x3, // position
    1 => Float32x3, // normal
    2 => Float32x2, // uv_0
    3 => Float32x2, // uv_1
    4 => Float32x4, // joint_0
    5 => Float32x4, // weight_0
    6 => Float32x3, // color_0
    ]
    .to_vec()
}

pub fn create_vertex_description(attributes: &[VertexAttribute]) -> wgpu::VertexBufferLayout {
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes,
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    position: glm::Vec3,
    padding_0: f32,
    color: glm::Vec3,
    padding_1: f32,
}

impl Light {
    pub fn new(position: glm::Vec3, color: glm::Vec3) -> Self {
        Self {
            position,
            padding_0: 0.0,
            color,
            padding_1: 0.0,
        }
    }
}

pub struct Geometry {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl Geometry {
    pub fn new<T: bytemuck::Pod>(device: &Device, vertices: &[T], indices: &[u32]) -> Self {
        Self {
            vertex_buffer: Self::create_vertex_buffer(device, vertices),
            index_buffer: Self::create_index_buffer(device, indices),
        }
    }

    pub fn slices(&self) -> (wgpu::BufferSlice, wgpu::BufferSlice) {
        (self.vertex_buffer.slice(..), self.index_buffer.slice(..))
    }

    fn create_vertex_buffer(device: &Device, vertices: &[impl bytemuck::Pod]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_index_buffer(device: &Device, indices: &[impl bytemuck::Pod]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }
}

pub struct UniformBinding {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl UniformBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Uniform Buffer Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Uniform Buffer Bind Group"),
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn upload_uniform_data(&self, queue: &Queue, offset: BufferAddress, data: Uniform) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&[data]));
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    pub view: glm::Mat4,
    pub projection: glm::Mat4,
    pub camera_position: glm::Vec4,
    pub light: Light,
}

pub struct DynamicUniformBinding {
    pub alignment: wgpu::BufferAddress,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DynamicUniformBinding {
    pub const MAX_NUMBER_OF_MESHES: usize = 10_000;

    pub fn new(device: &wgpu::Device) -> Self {
        let alignment = device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Uniform Buffer"),
            size: (Self::MAX_NUMBER_OF_MESHES as wgpu::BufferAddress) * alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size_of::<DynamicUniform>() as _),
                },
                count: None,
            }],
            label: Some("Dynamic Uniform Buffer Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(size_of::<DynamicUniform>() as _),
                }),
            }],
            label: Some("World Uniform Buffer Bind Group"),
        });

        Self {
            alignment,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn upload_uniform_data(&self, queue: &Queue, offset: BufferAddress, data: &[impl Copy]) {
        queue.write_buffer(&self.buffer, offset, unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * self.alignment as usize,
            )
        });
    }
}

#[repr(C, align(256))]
#[derive(Default, Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct DynamicUniform {
    pub model: glm::Mat4,
}

const SHADER_SOURCE: &str = "
struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
};

struct Uniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    light: Light,
};

@group(0) @binding(0)
var<uniform> ubo: Uniform;

struct DynamicUniform {
    model: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> mesh_ubo: DynamicUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv_0: vec2<f32>,
    @location(3) uv_1: vec2<f32>,
    @location(4) joint_0: vec4<f32>,
    @location(5) weight_0: vec4<f32>,
    @location(6) color_0: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vertex_main(vert: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let mvp = ubo.projection * ubo.view * mesh_ubo.model;
    out.position = mvp * vec4(vert.position, 1.0);
    out.normal = vec4((mvp * vec4(vert.normal, 0.0)).xyz, 1.0).xyz;
    return out;
};

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = vec4(0.2, 0.3, 0.4, 1.0);

    let ambient_strength = 0.1;
    let ambient_color = ubo.light.color.rgb * ambient_strength;

    let light_dir = normalize(in.position.xyz - ubo.light.position.xyz);
    let diffuse_strength = max(dot(in.normal, light_dir), 0.0);
    let diffuse_color = ubo.light.color.rgb * diffuse_strength;

    let view_dir = normalize(ubo.camera_position.xyz - in.position.xyz);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * ubo.light.color.rgb;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.rgb;

    return vec4<f32>(result, object_color.a);
}
";
