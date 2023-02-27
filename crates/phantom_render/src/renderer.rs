use crate::backend::{VulkanRenderer, WgpuRenderer};
use phantom_config::Config;
use phantom_gui::GuiFrame;
use phantom_world::{Viewport, World};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum Backend {
    Dx11,
    Dx12,
    Metal,
    Vulkan,
    VulkanWgpu,
}

pub trait Renderer {
    fn load_world(&mut self, world: &World) -> Result<(), Box<dyn Error>>;
    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn Error>>;
    fn render_frame(
        &mut self,
        world: &mut World,
        config: &Config,
        gui_frame: &mut GuiFrame,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn create_renderer<W: HasRawWindowHandle + HasRawDisplayHandle>(
    backend: &Backend,
    window_handle: &W,
    viewport: &Viewport,
) -> Result<Box<dyn Renderer>, Box<dyn Error>> {
    let backend = if let Backend::Vulkan = backend {
        Box::new(VulkanRenderer::new(&window_handle, viewport)?) as _
    } else {
        Box::new(WgpuRenderer::new(&window_handle, backend, viewport)?) as _
    };
    Ok(backend)
}
