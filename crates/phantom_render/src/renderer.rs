use crate::backend::{VulkanRenderer, WgpuRenderer};
use egui::ClippedPrimitive;
use egui_wgpu::renderer::ScreenDescriptor;
use phantom_config::Config;
use phantom_gui::GuiFrameResources;
use phantom_world::{Viewport, World};
use raw_window_handle::HasRawWindowHandle;
use std::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum Backend {
    Dx11Wgpu,
    Dx12Wgpu,
    MetalWgpu,
    VulkanWgpu,
    Vulkan,
}

pub trait Renderer {
    fn load_world(&mut self, world: &World) -> Result<(), Box<dyn Error>>;
    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn Error>>;
    fn update(
        &mut self,
        world: &mut World,
        config: &Config,
        gui_frame_resources: &mut GuiFrameResources,
    ) -> Result<(), Box<dyn Error>>;
    fn render_frame(
        &mut self,
        world: &mut World,
        config: &Config,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn create_renderer(
    backend: &Backend,
    window_handle: &impl HasRawWindowHandle,
    viewport: &Viewport,
) -> Result<Box<dyn Renderer>, Box<dyn Error>> {
    let renderer: Box<dyn Renderer> = match backend {
        Backend::Dx11Wgpu | Backend::Dx12Wgpu | Backend::MetalWgpu | Backend::VulkanWgpu => {
            let renderer = WgpuRenderer::new(&window_handle, backend, viewport)?;
            Box::new(renderer) as _
        }
        Backend::Vulkan => {
            let renderer = VulkanRenderer::new(&window_handle, *viewport)?;
            Box::new(renderer) as _
        }
    };
    Ok(renderer)
}
