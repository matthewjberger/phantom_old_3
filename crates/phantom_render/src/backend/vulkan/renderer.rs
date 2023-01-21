use phantom_vulkan::core::{Context, Frame};
use phantom_world::Viewport;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;
use thiserror::Error;

use crate::Renderer;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create Vulkan context")]
    CreateContext(#[from] anyhow::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) struct VulkanRenderer {
    viewport: Viewport,
    frame: Frame,
    context: Arc<Context>,
}

impl VulkanRenderer {
    const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window_handle: &W,
        viewport: &Viewport,
    ) -> Result<Self> {
        let context = Arc::new(Context::new(window_handle).map_err(Error::CreateContext)?);
        let frame = Frame::new(context.clone(), *viewport, Self::MAX_FRAMES_IN_FLIGHT)?;
        let renderer = Self {
            viewport: *viewport,
            frame,
            context,
        };
        Ok(renderer)
    }
}

impl Renderer for VulkanRenderer {
    fn load_world(
        &mut self,
        world: &phantom_world::World,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn resize(
        &mut self,
        dimensions: [u32; 2],
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn update(
        &mut self,
        world: &mut phantom_world::World,
        config: &phantom_config::Config,
        gui_frame_resources: &mut phantom_gui::GuiFrameResources,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn render_frame(
        &mut self,
        world: &mut phantom_world::World,
        config: &phantom_config::Config,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::renderer::ScreenDescriptor,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
