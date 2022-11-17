use std::sync::Arc;

use anyhow::Result;
use phantom_config::Config;
use phantom_vulkan::{
    ash::vk::{self, CommandBuffer},
    core::{
        CommandPool, Context, Device, Image, ImageNode, RawImage, RenderGraph, ShaderCache,
        ShaderPathSetBuilder, Swapchain, SwapchainProperties,
    },
    render::{FullscreenRender, FullscreenUniformBuffer},
};
use phantom_world::World;

pub(crate) struct SceneRender {
    pub fullscreen_pipeline: Option<FullscreenRender>,
    pub rendergraph: RenderGraph,
    pub shader_cache: ShaderCache,
    pub samples: vk::SampleCountFlags,
    context: Arc<Context>,
}

impl SceneRender {
    pub fn new(
        context: Arc<Context>,
        swapchain: &Swapchain,
        swapchain_properties: &SwapchainProperties,
    ) -> Result<Self> {
        let samples = context.max_usable_samples();
        let rendergraph =
            Self::create_rendergraph(&context, swapchain, swapchain_properties, samples)?;
        let shader_cache = ShaderCache::default();

        let mut scene = Self {
            fullscreen_pipeline: None,
            rendergraph,
            shader_cache,
            samples,
            context,
        };
        scene.create_pipelines()?;
        Ok(scene)
    }

    pub fn create_pipelines(&mut self) -> Result<()> {
        let fullscreen_pass = self.rendergraph.pass_handle("fullscreen")?;

        let shader_path_set = ShaderPathSetBuilder::default()
            .vertex("assets/shaders/postprocessing/fullscreen_triangle.vert.spv")
            .fragment("assets/shaders/postprocessing/postprocess.frag.spv")
            .build()?;

        self.fullscreen_pipeline = None;
        let fullscreen_pipeline = FullscreenRender::new(
            &self.context,
            fullscreen_pass.clone(),
            &mut self.shader_cache,
            self.rendergraph.image_view("color_resolve")?.handle,
            self.rendergraph.sampler("default")?.handle,
            shader_path_set,
        )?;
        self.fullscreen_pipeline = Some(fullscreen_pipeline);

        Ok(())
    }

    pub fn create_rendergraph(
        context: &Context,
        swapchain: &Swapchain,
        swapchain_properties: &SwapchainProperties,
        samples: vk::SampleCountFlags,
    ) -> Result<RenderGraph> {
        let device = context.device.clone();
        let allocator = context.allocator.clone();

        let offscreen = "offscreen";
        let fullscreen = "fullscreen";
        let color = "color";
        let color_resolve = "color_resolve";
        let offscreen_extent = vk::Extent2D::builder().width(2048).height(2048).build();
        let mut rendergraph = RenderGraph::new(
            &[offscreen, fullscreen],
            vec![
                ImageNode {
                    name: color.to_string(),
                    extent: offscreen_extent,
                    format: vk::Format::R8G8B8A8_UNORM,
                    clear_value: vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.39, 0.58, 0.92, 1.0],
                        },
                    },
                    samples,
                    force_store: false,
                    force_shader_read: false,
                },
                ImageNode {
                    name: RenderGraph::DEPTH_STENCIL.to_owned(),
                    extent: offscreen_extent,
                    format: vk::Format::D24_UNORM_S8_UINT,
                    clear_value: vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    },
                    samples,
                    force_store: false,
                    force_shader_read: false,
                },
                ImageNode {
                    name: color_resolve.to_string(),
                    extent: offscreen_extent,
                    format: vk::Format::R8G8B8A8_UNORM,
                    clear_value: vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [1.0, 1.0, 1.0, 1.0],
                        },
                    },
                    samples: vk::SampleCountFlags::TYPE_1,
                    force_store: false,
                    force_shader_read: false,
                },
                ImageNode {
                    name: RenderGraph::backbuffer_name(0),
                    extent: swapchain_properties.extent,
                    format: swapchain_properties.surface_format.format,
                    clear_value: vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [1.0, 1.0, 1.0, 1.0],
                        },
                    },
                    samples: vk::SampleCountFlags::TYPE_1,
                    force_store: false,
                    force_shader_read: false,
                },
            ],
            &[
                (offscreen, color),
                (offscreen, color_resolve),
                (offscreen, RenderGraph::DEPTH_STENCIL),
                (color_resolve, fullscreen),
                (fullscreen, &RenderGraph::backbuffer_name(0)),
            ],
        )?;

        rendergraph.build(device.clone(), allocator)?;

        rendergraph.print_graph();

        let swapchain_images = swapchain
            .images()?
            .into_iter()
            .map(|handle| Box::new(RawImage(handle)) as Box<dyn Image>)
            .collect::<Vec<_>>();
        rendergraph.insert_backbuffer_images(device, swapchain_images)?;

        Ok(rendergraph)
    }

    pub fn load_world(&mut self, _world: &World) -> Result<()> {
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        Ok(())
    }

    pub fn execute_passes(&self, command_buffer: CommandBuffer, image_index: usize) -> Result<()> {
        let device = &self.context.device.clone();
        self.rendergraph.execute_pass(
            command_buffer,
            "offscreen",
            image_index,
            |pass, command_buffer| {
                device.update_viewport(command_buffer, pass.extent, true)?;
                Ok(())
            },
        )?;

        self.rendergraph.execute_pass(
            command_buffer,
            "fullscreen",
            image_index,
            |pass, command_buffer| {
                device.update_viewport(command_buffer, pass.extent, false)?;
                if let Some(fullscreen_pipeline) = self.fullscreen_pipeline.as_ref() {
                    fullscreen_pipeline.issue_commands(command_buffer)?;
                }
                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn recreate_rendergraph(
        &mut self,
        swapchain: &Swapchain,
        swapchain_properties: &SwapchainProperties,
    ) -> Result<()> {
        let rendergraph = SceneRender::create_rendergraph(
            &self.context,
            swapchain,
            swapchain_properties,
            self.samples,
        )?;
        self.rendergraph = rendergraph;
        self.create_pipelines()?;
        Ok(())
    }
}
