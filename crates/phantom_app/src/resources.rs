mod input;
mod system;

pub use self::{input::*, system::*};

use phantom_world::World;
use phantom_dependencies::{gilrs::Gilrs, winit::window::Window};
use phantom_gui::Gui;

pub struct Resources<'a> {
    pub world: &'a mut World,
    pub window: &'a mut Window,
    pub gui: &'a mut Gui,
    pub gilrs: &'a mut Gilrs,
    pub input: &'a mut Input,
    pub system: &'a mut System,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
