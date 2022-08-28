mod input;
mod system;

use std::path::Path;

pub use self::{input::*, system::*};

use phantom_dependencies::{
    gilrs::Gilrs,
    glutin::{ContextWrapper, PossiblyCurrent},
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

    #[error("Failed to reset world!")]
    ResetWorld(#[source] WorldError),

    #[error("Failed to load gltf asset!")]
    LoadGltfAsset(#[source] GltfError),

    #[error("Failed to sync renderer with world!")]
    SyncRenderer(#[source] Box<dyn std::error::Error>),
}

type Result<T, E = ResourceError> = std::result::Result<T, E>;

pub struct Resources<'a> {
    pub renderer: &'a mut Box<dyn Renderer>,
    pub world: &'a mut World,
    pub context: &'a mut ContextWrapper<PossiblyCurrent, Window>,
    pub gui: &'a mut Gui,
    pub gilrs: &'a mut Gilrs,
    pub input: &'a mut Input,
    pub system: &'a mut System,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_grab(&mut self, grab: CursorGrabMode) -> Result<()> {
        self.context
            .window()
            .set_cursor_grab(grab)
            .map_err(ResourceError::SetCursorGrabMode)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.context.window().set_cursor_visible(visible)
    }

    pub fn center_cursor(&mut self) -> Result<()> {
        self.set_cursor_position(&self.system.window_center())
    }

    pub fn set_cursor_position(&mut self, position: &glm::Vec2) -> Result<()> {
        self.context
            .window()
            .set_cursor_position(PhysicalPosition::new(position.x, position.y))
            .map_err(ResourceError::SetCursorPosition)
    }

    pub fn set_fullscreen(&mut self) {
        self.context
            .window()
            .set_fullscreen(Some(Fullscreen::Borderless(
                self.context.window().primary_monitor(),
            )));
    }

    pub fn close_map(&mut self) -> Result<()> {
        *self.world = World::new().map_err(ResourceError::ResetWorld)?;
        self.renderer
            .sync_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }

    pub fn load_map(&mut self, path: impl AsRef<Path>) -> Result<()> {
        *self.world = World::load(path).map_err(ResourceError::LoadMap)?;
        self.renderer
            .sync_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }

    pub fn load_gltf_asset(&mut self, path: impl AsRef<Path>) -> Result<()> {
        load_gltf(path, self.world).map_err(ResourceError::LoadGltfAsset)?;
        log::info!("Loaded gltf asset");
        self.renderer
            .sync_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }
}
