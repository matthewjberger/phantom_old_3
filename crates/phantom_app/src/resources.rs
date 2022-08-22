mod input;
mod system;

use std::path::Path;

pub use self::{input::*, system::*};

use phantom_dependencies::{
    gilrs::Gilrs,
    legion::world::EntityAccessError,
    log, nalgebra_glm as glm,
    thiserror::Error,
    winit::{
        dpi::PhysicalPosition,
        error::ExternalError,
        window::{CursorGrabMode, Fullscreen, Window},
    },
};
use phantom_gui::Gui;
use phantom_render::Renderer;
use phantom_world::{load_gltf, GltfError, World, WorldError};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Failed to access entity!")]
    AccessEntity(#[from] EntityAccessError),

    #[error("Failed to set cursor grab mode!")]
    SetCursorGrabMode(#[source] ExternalError),

    #[error("Failed to set cursor position!")]
    SetCursorPosition(#[source] ExternalError),

    #[error("Failed to load map!")]
    LoadMap(#[source] WorldError),

    #[error("Failed to load gltf asset!")]
    LoadGltfAsset(#[source] GltfError),
}

type Result<T, E = ResourceError> = std::result::Result<T, E>;

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
        self.window
            .set_cursor_grab(grab)
            .map_err(ResourceError::SetCursorGrabMode)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }

    pub fn center_cursor(&mut self) -> Result<()> {
        Ok(self.set_cursor_position(&self.system.window_center())?)
    }

    pub fn set_cursor_position(&mut self, position: &glm::Vec2) -> Result<()> {
        self.window
            .set_cursor_position(PhysicalPosition::new(position.x, position.y))
            .map_err(ResourceError::SetCursorPosition)
    }

    pub fn set_fullscreen(&mut self) {
        self.window
            .set_fullscreen(Some(Fullscreen::Borderless(self.window.primary_monitor())));
    }

    pub fn load_map(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.world.reload(path).map_err(ResourceError::LoadMap)?;
        // TODO: Update renderer here
        Ok(())
    }

    pub fn load_gltf_asset(&mut self, path: impl AsRef<Path>) -> Result<()> {
        load_gltf(path, &mut self.world).map_err(ResourceError::LoadGltfAsset)?;
        // TODO: Update renderer here
        log::info!("Loaded gltf asset");
        Ok(())
    }
}
