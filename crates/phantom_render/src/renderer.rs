use crate::backend::{OpenGlRenderer, WgpuRenderer};
use phantom_dependencies::{
    egui::ClippedPrimitive,
    egui_wgpu::renderer::ScreenDescriptor,
    glutin::{ContextWrapper, PossiblyCurrent},
    winit::window::Window,
};
use phantom_gui::GuiFrameResources;
use phantom_world::{Viewport, World};

pub enum Backend {
    // TODO: Route specific backends through here
    Wgpu,
    OpenGL,
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
        Backend::Wgpu => {
            let window_handle = context.window();
            let backend = WgpuRenderer::new(window_handle, viewport)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
        Backend::OpenGL => {
            let backend = OpenGlRenderer::new(context, viewport)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
