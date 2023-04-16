use phantom_config::Config;
use phantom_gui::GuiFrame;
use phantom_world::World;
use std::error::Error;

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
