use super::wgpu::WgpuRenderer;
use phantom_dependencies::{
    egui::ClippedPrimitive, egui_wgpu::renderer::ScreenDescriptor,
    raw_window_handle::HasRawWindowHandle,
};
use phantom_gui::GuiFrameResources;
use phantom_world::{Viewport, World};

pub enum Backend {
    // TODO: Route specific backends through here
    Wgpu,
    // OpenGl,
}

pub trait Renderer {
    fn sync_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>>;
    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn std::error::Error>>;
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
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn create_renderer(
    backend: &Backend,
    window_handle: &impl HasRawWindowHandle,
    viewport: &Viewport,
) -> Result<Box<dyn Renderer>, Box<dyn std::error::Error>> {
    match backend {
        Backend::Wgpu => {
            let backend = WgpuRenderer::new(window_handle, viewport)?;
            Ok(Box::new(backend) as Box<dyn Renderer>)
        }
    }
}
