mod input;
mod system;

pub use self::{input::*, system::*};

use phantom_dependencies::{
    anyhow::Result,
    gilrs::Gilrs,
    nalgebra_glm as glm,
    winit::{
        dpi::PhysicalPosition,
        window::{CursorGrabMode, Fullscreen, Window},
    },
};
use phantom_gui::Gui;
use phantom_render::Renderer;
use phantom_world::World;

pub struct Resources<'a> {
    pub renderer: &'a mut Renderer,
    pub world: &'a mut World,
    pub window: &'a mut Window,
    pub gui: &'a mut Gui,
    pub gilrs: &'a mut Gilrs,
    pub input: &'a mut Input,
    pub system: &'a mut System,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_grab(&mut self, grab: CursorGrabMode) -> Result<()> {
        Ok(self.window.set_cursor_grab(grab)?)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }

    pub fn center_cursor(&mut self) -> Result<()> {
        Ok(self.set_cursor_position(&self.system.window_center())?)
    }

    pub fn set_cursor_position(&mut self, position: &glm::Vec2) -> Result<()> {
        Ok(self
            .window
            .set_cursor_position(PhysicalPosition::new(position.x, position.y))?)
    }

    pub fn set_fullscreen(&mut self) {
        self.window
            .set_fullscreen(Some(Fullscreen::Borderless(self.window.primary_monitor())));
    }
}
