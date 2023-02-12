use crate::{
    vulkan::core::{Context, Frame},
    Renderer,
};
use anyhow::Result;
use phantom_world::Viewport;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;

pub struct VulkanDevice {
    viewport: Viewport,
    frame: Frame,
    context: Arc<Context>,
}

impl VulkanDevice {
    const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window_handle: &W,
        viewport: Viewport,
    ) -> Result<Self> {
        let context = Arc::new(Context::new(window_handle)?);
        let frame = Frame::new(context.clone(), viewport, Self::MAX_FRAMES_IN_FLIGHT)?;
        log::info!("Created Vulkan render device successfully!");
        Ok(Self {
            viewport,
            frame,
            context,
        })
    }
}

impl Renderer for VulkanDevice {
    fn load_world(
        &mut self,
        _world: &phantom_world::World,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn resize(&mut self, _dimensions: [u32; 2]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn render_frame(
        &mut self,
        _world: &mut phantom_world::World,
        _config: &phantom_config::Config,
        _gui_frame: &mut phantom_gui::GuiFrame,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Self { frame, .. } = self;

        let _aspect_ratio = frame.swapchain_properties.aspect_ratio();
        let viewport = self.viewport;
        frame.render(viewport, |_command_buffer, _image_index| {
            // TODO: add scene rendering
            Ok(())
        })?;

        if frame.recreated_swapchain {
            // TODO: recreate swapchain
        }

        Ok(())
    }
}
