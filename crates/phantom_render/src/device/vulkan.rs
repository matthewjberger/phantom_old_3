use crate::Renderer;

#[derive(Default)]
pub struct VulkanDevice;

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
        Ok(())
    }
}
