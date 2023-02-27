use phantom_render_traits::GpuDevice;
use phantom_world::Viewport;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("No suitable GPU adapters found on the system!")]
    // NoSuitableGpuAdapters,

    // #[error("Failed to find a support swapchain format!")]
    // NoSupportedSwapchainFormat,
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct VulkanGpuDevice;

impl VulkanGpuDevice {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window_handle: &W,
        viewport: &Viewport,
    ) -> Result<Self> {
        pollster::block_on(Self::new_async(window_handle, viewport))
    }

    async fn new_async<W: HasRawWindowHandle + HasRawDisplayHandle>(
        _window_handle: &W,
        _viewport: &Viewport,
    ) -> Result<Self> {
        Ok(Self {})
    }
}

impl GpuDevice for VulkanGpuDevice {
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
        Ok(())
    }
}
