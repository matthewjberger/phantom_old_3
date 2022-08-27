use super::pbr::PbrShader;
use crate::backend::opengl::{
    buffer::GeometryBuffer,
    graphics::{BlendFunction, CullMode, DepthTestFunction, FrontFace, Graphics},
    texture::Texture,
};
use phantom_dependencies::{anyhow::Result, gl, legion::EntityStore, petgraph::graph::NodeIndex};
use phantom_world::{AlphaMode, Format, Material, MeshRender, SceneGraph, World};
use std::ptr;

impl From<&phantom_world::Texture> for Texture {
    fn from(world_texture: &phantom_world::Texture) -> Self {
        let pixel_format = match world_texture.format {
            Format::R8 => gl::R8,
            Format::R8G8 => gl::RG,
            Format::R8G8B8 => gl::RGB,
            Format::R8G8B8A8 => gl::RGBA,
            Format::B8G8R8 => gl::BGR,
            Format::B8G8R8A8 => gl::BGRA,
            Format::R16 => gl::R16,
            Format::R16G16 => gl::RG16,
            Format::R16G16B16 => gl::RGB16,
            Format::R16G16B16A16 => gl::RGBA16,
            Format::R16F => gl::R16F,
            Format::R16G16F => gl::RG16F,
            Format::R16G16B16F => gl::RGB16F,
            Format::R16G16B16A16F => gl::RGBA16F,
            Format::R32 => gl::R32UI,
            Format::R32G32 => gl::RG32UI,
            Format::R32G32B32 => gl::RGB32UI,
            Format::R32G32B32A32 => gl::RGBA32UI,
            Format::R32F => gl::R32F,
            Format::R32G32F => gl::RG32F,
            Format::R32G32B32F => gl::RGB32F,
            Format::R32G32B32A32F => gl::RGBA32F,
        };

        let mut texture = Texture::new();
        texture.load_data(
            world_texture.width,
            world_texture.height,
            &world_texture.pixels,
            pixel_format,
        );
        texture
    }
}

pub struct WorldRender {
    pub geometry: GeometryBuffer,
    pub pbr_shader: PbrShader,
    pub textures: Vec<Texture>,
}

impl WorldRender {
    pub fn new(world: &World) -> Result<Self> {
        let geometry = GeometryBuffer::new(
            &world.geometry.vertices,
            Some(&world.geometry.indices),
            &[3, 3, 2, 2, 4, 4, 3],
        );

        let pbr_shader = PbrShader::new()?;

        let textures = world
            .textures
            .iter()
            .map(|x| Texture::from(x))
            .collect::<Vec<_>>();

        Ok(Self {
            geometry,
            pbr_shader,
            textures,
        })
    }

    pub fn render(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        Graphics::enable_culling(CullMode::Back, FrontFace::CounterClockwise);
        Graphics::enable_depth_testing(DepthTestFunction::LessThanOrEqualTo);

        self.geometry.bind();
        self.pbr_shader.update(world, aspect_ratio)?;

        for alpha_mode in [AlphaMode::Opaque, AlphaMode::Mask, AlphaMode::Blend].iter() {
            for graph in world.scene.graphs.iter() {
                graph
                    .walk(|node_index| Ok(self.visit_node(node_index, graph, world, alpha_mode)?))
                    .unwrap();
            }
        }

        Ok(())
    }

    fn visit_node(
        &self,
        node_index: NodeIndex,
        graph: &SceneGraph,
        world: &World,
        alpha_mode: &AlphaMode,
    ) -> Result<()> {
        let entity = graph[node_index];

        let model = world.global_transform(graph, node_index).unwrap();

        self.pbr_shader.update_model_matrix(model);

        match world
            .ecs
            .entry_ref(entity)
            .unwrap()
            .get_component::<MeshRender>()
        {
            Ok(mesh_render) => {
                if let Some(mesh) = world.geometry.meshes.get(&mesh_render.name) {
                    match alpha_mode {
                        AlphaMode::Opaque | AlphaMode::Mask => Graphics::disable_blending(),
                        AlphaMode::Blend => Graphics::enable_blending(
                            BlendFunction::SourceAlpha,
                            BlendFunction::OneMinusSourceAlpha,
                        ),
                    }

                    for primitive in mesh.primitives.iter() {
                        let material = match primitive.material_index {
                            Some(material_index) => {
                                let primitive_material =
                                    world.material_at_index(material_index).unwrap();
                                if primitive_material.alpha_mode != *alpha_mode {
                                    continue;
                                }
                                primitive_material.clone()
                            }
                            None => Material::default(),
                        };

                        self.pbr_shader.update_material(&material, &self.textures)?;

                        let ptr: *const u8 = ptr::null_mut();
                        let ptr =
                            unsafe { ptr.add(primitive.first_index * std::mem::size_of::<u32>()) };
                        unsafe {
                            gl::DrawElements(
                                gl::TRIANGLES,
                                primitive.number_of_indices as _,
                                gl::UNSIGNED_INT,
                                ptr as *const _,
                            );
                        }
                    }
                }
            }
            Err(_) => return Ok(()),
        }

        Ok(())
    }
}
