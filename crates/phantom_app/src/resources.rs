mod input;
mod system;

pub use self::{input::*, system::*};

use gilrs::Gilrs;
use legion::world::EntityAccessError;
use nalgebra_glm as glm;
use phantom_config::Config;
use phantom_gui::Gui;
use phantom_render::Renderer;
use phantom_world::{load_gltf, GltfError, RaycastConfiguration, World, WorldError};
use std::path::Path;
use thiserror::Error;
use winit::{
    dpi::PhysicalPosition,
    error::ExternalError,
    window::{CursorGrabMode, Fullscreen, Window},
};

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

    #[error("Failed to create raycast configuration!")]
    Raycast(#[from] WorldError),
}

type Result<T, E = ResourceError> = std::result::Result<T, E>;

pub struct Resources<'a> {
    pub config: &'a mut Config,
    pub window: &'a mut Window,
    pub gilrs: &'a mut Gilrs,
    pub gui: &'a mut Gui,
    pub input: &'a mut Input,
    pub renderer: &'a mut Box<dyn Renderer>,
    pub system: &'a mut System,
    pub world: &'a mut World,
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
        self.set_cursor_position(&self.system.window_center())
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

    pub fn close_map(&mut self) -> Result<()> {
        *self.world = World::new().map_err(ResourceError::ResetWorld)?;
        self.renderer
            .load_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }

    pub fn open_map(&mut self, path: impl AsRef<Path>) -> Result<()> {
        *self.world = World::load(path).map_err(ResourceError::LoadMap)?;
        self.renderer
            .load_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }

    pub fn load_gltf(&mut self, path: impl AsRef<Path>) -> Result<()> {
        load_gltf(path, self.world).map_err(ResourceError::LoadGltfAsset)?;
        log::info!("Loaded gltf asset");
        self.renderer
            .load_world(self.world)
            .map_err(ResourceError::SyncRenderer)
    }

    pub fn raycast_configuration(&self) -> Result<RaycastConfiguration> {
        let viewport = self.renderer.viewport();

        let (projection, view) = self
            .world
            .active_camera_matrices(viewport.aspect_ratio())
            .map_err(ResourceError::Raycast)?;

        let mouse_ray_configuration = RaycastConfiguration {
            viewport,
            projection_matrix: projection,
            view_matrix: view,
            mouse_position: self.input.mouse.position,
        };

        Ok(mouse_ray_configuration)
    }
}
