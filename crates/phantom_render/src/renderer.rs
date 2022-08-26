use crate::backend::{OpenGlRenderer, WgpuRenderer};
use phantom_dependencies::{
    egui::ClippedPrimitive,
    egui_wgpu::renderer::ScreenDescriptor,
    glutin::{ContextWrapper, PossiblyCurrent},
    winit::window::Window,
};
use phantom_gui::GuiFrameResources;
use phantom_world::{Viewport, World};

#[derive(Debug, Copy, Clone)]
pub enum Backend {
    Dx11,
    Dx12,
    Metal,
    OpenGL,
    Vulkan,
}

pub trait Renderer {
    fn sync_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>>;
    fn resize(
        &mut self,
        dimensions: [u32; 2],
        context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn update(
        &mut self,
        world: &mut World,
        gui_frame_resources: &mut GuiFrameResources,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn render_frame(
        &mut self,
        world: &mut World,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
        context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn create_renderer(
    backend: &Backend,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    viewport: &Viewport,
) -> Result<Box<dyn Renderer>, Box<dyn std::error::Error>> {
    match backend {
        Backend::OpenGL => {
            let backend = OpenGlRenderer::new(context, viewport)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
        backend => {
            let window_handle = context.window();
            let backend = WgpuRenderer::new(window_handle, backend, viewport)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
