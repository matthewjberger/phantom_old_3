use crate::device::DummyDevice;
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
}

pub trait GpuDevice {
    fn load_world(&mut self, world: &World) -> Result<(), Box<dyn Error>>;
    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn Error>>;
    fn render_frame(
        &mut self,
        world: &mut World,
        config: &Config,
        gui_frame: &mut GuiFrame,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn create_gpu_device<W: HasRawWindowHandle + HasRawDisplayHandle>(
    _backend: &Backend,
    _window_handle: &W,
    _viewport: &Viewport,
) -> Result<Box<dyn GpuDevice>, Box<dyn Error>> {
    let backend = DummyDevice::default();
    Ok(Box::new(backend) as Box<dyn GpuDevice>)
}
