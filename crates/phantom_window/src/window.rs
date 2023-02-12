use image::io::Reader;
use std::io;
use thiserror::Error;
use winit::{
    self,
    dpi::PhysicalSize,
    error::OsError,
    event_loop::EventLoop,
    window::{Icon, WindowBuilder},
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create icon file!")]
    CreateIcon(#[source] winit::window::BadIcon),

    #[error("Failed to create a window!")]
    CreateWindow(#[source] OsError),

    #[error("Failed to decode icon file at path: {1}")]
    DecodeIconFile(#[source] image::ImageError, String),

    #[error("Failed to handle an event in the state machine!")]
    HandleEvent(#[source] Box<dyn std::error::Error>),

    #[error("Failed to open icon file at path: {1}")]
    OpenIconFile(#[source] io::Error, String),

    #[error("Failed to render a frame!")]
    RenderFrame(#[source] Box<dyn std::error::Error>),

    #[error("Failed to start the state machine!")]
    StartStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to stop the state machine!")]
    StopStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to update the state machine!")]
    UpdateStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to to update the gui!")]
    UpdateGui(#[source] Box<dyn std::error::Error>),

    #[error("Failed to to resize the renderer!")]
    ResizeRenderer(#[source] Box<dyn std::error::Error>),
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub title: String,
    pub icon: Option<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            is_fullscreen: false,
            title: "Phantom App".to_string(),
            icon: None,
        }
    }
}

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

impl Window {
    pub fn new(config: WindowConfig) -> Result<Self> {
        let event_loop = EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title(config.title.to_string())
            .with_inner_size(PhysicalSize::new(config.width, config.height));

        if let Some(icon_path) = config.icon.as_ref() {
            let image = Reader::open(icon_path)
                .map_err(|error| Error::OpenIconFile(error, icon_path.to_string()))?
                .decode()
                .map_err(|error| Error::DecodeIconFile(error, icon_path.to_string()))?
                .into_rgba8();
            let (width, height) = image.dimensions();
            let icon =
                Icon::from_rgba(image.into_raw(), width, height).map_err(Error::CreateIcon)?;
            window_builder = window_builder.with_window_icon(Some(icon));
        }

        let window = window_builder
            .build(&event_loop)
            .map_err(Error::CreateWindow)?;

        Ok(Self { window, event_loop })
    }
}
