use super::scene::SceneRender;
use crate::Renderer;
use anyhow::Result;
use phantom_vulkan::core::{Context, Frame};
use phantom_world::{Viewport, World};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub struct VulkanRenderer {
    viewport: Viewport,
    frame: Frame,
    context: Arc<Context>,
    scene_render: SceneRender,
}

impl VulkanRenderer {
    const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(window_handle: &impl HasRawWindowHandle, viewport: Viewport) -> Result<Self> {
        let context = Arc::new(Context::new(window_handle)?);
        let frame = Frame::new(context.clone(), viewport, Self::MAX_FRAMES_IN_FLIGHT)?;
        let scene_render = SceneRender::new(
            context.clone(),
            frame.swapchain()?,
            &frame.swapchain_properties,
        )?;
        let renderer = Self {
            viewport,
            frame,
            context,
            scene_render,
        };
        Ok(renderer)
    }
}

impl Renderer for VulkanRenderer {
    fn load_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        self.scene_render.load_world(world)?;
        Ok(())
    }

    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn std::error::Error>> {
        self.viewport = Viewport::from(dimensions);
        Ok(())
    }

    fn update(
        &mut self,
        world: &mut phantom_world::World,
        config: &phantom_config::Config,
        gui_frame_resources: &mut phantom_gui::GuiFrameResources,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let aspect_ratio = self.frame.swapchain_properties.aspect_ratio();
        self.scene_render.update()?;
        Ok(())
    }

    fn render_frame(
        &mut self,
        _world: &mut phantom_world::World,
        _config: &phantom_config::Config,
        _paint_jobs: &[egui::ClippedPrimitive],
        _screen_descriptor: &egui_wgpu::renderer::ScreenDescriptor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Self {
            frame,
            scene_render: scene,
            ..
        } = self;

        let viewport = self.viewport;
        frame.render(viewport, |command_buffer, image_index| {
            scene.execute_passes(command_buffer, image_index)
        })?;

        if frame.recreated_swapchain {
            scene.recreate_rendergraph(frame.swapchain()?, &frame.swapchain_properties)?;
        }

        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Err(error) = self.context.device.handle.device_wait_idle() {
                log::error!("{}", error);
            }
        }
    }
}
