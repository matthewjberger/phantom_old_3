use crate::{Input, Resources, State, StateMachine, System};
use phantom_dependencies::{
    anyhow::{self, anyhow},
    env_logger,
    gilrs::Gilrs,
    image::{self, io::Reader},
    log,
    thiserror::Error,
    winit::{
        self,
        dpi::PhysicalSize,
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Fullscreen, Icon, WindowBuilder},
    },
};

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    BadIcon(#[from] winit::window::BadIcon),
    #[error(transparent)]
    OsError(#[from] winit::error::OsError),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("Unknown application error")]
    Unknown,
}

type Result<T, E = ApplicationError> = std::result::Result<T, E>;

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub title: String,
    pub icon: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            is_fullscreen: false,
            title: "Phantom Editor".to_string(),
            icon: None,
        }
    }
}

pub fn run(initial_state: impl State + 'static, config: AppConfig) -> Result<()> {
    env_logger::init();
    log::info!("Phantom app started");

    let event_loop = EventLoop::new();
    let mut window_builder = WindowBuilder::new()
        .with_title(config.title.to_string())
        .with_inner_size(PhysicalSize::new(config.width, config.height));

    if let Some(icon_path) = config.icon.as_ref() {
        let image = Reader::open(icon_path)?.decode()?.into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height)?;
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let mut window = window_builder.build(&event_loop)?;

    if config.is_fullscreen {
        window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
    }

    let mut state_machine = StateMachine::new(initial_state);

    let physical_size = window.inner_size();
    let window_dimensions = [physical_size.width, physical_size.height];

    let mut gilrs = Gilrs::new().map_err(|_err| anyhow!("Failed to setup gamepad library!"))?;

    let mut input = Input::default();
    let mut system = System::new(window_dimensions);

    event_loop.run(move |event, _, control_flow| {
        let resources = Resources {
            window: &mut window,
            gilrs: &mut gilrs,
            input: &mut input,
            system: &mut system,
        };
        if let Err(error) = run_loop(&mut state_machine, &event, control_flow, resources) {
            log::error!("Application error: {}", error);
        }
    });
}

fn run_loop(
    state_machine: &mut StateMachine,
    event: &Event<()>,
    control_flow: &mut ControlFlow,
    mut resources: Resources,
) -> Result<()> {
    if !state_machine.is_running() {
        state_machine.start(&mut resources)?;
    }

    state_machine.handle_event(&mut resources, event)?;

    if let Some(event) = resources.gilrs.next_event() {
        state_machine.on_gamepad_event(&mut resources, event)?;
    }

    match event {
        Event::MainEventsCleared => {
            state_machine.update(&mut resources)?;
        }

        Event::WindowEvent {
            ref event,
            window_id,
        } if *window_id == resources.window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            WindowEvent::KeyboardInput { input, .. } => {
                if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
                    (input.virtual_keycode, input.state)
                {
                    *control_flow = ControlFlow::Exit;
                }

                state_machine.on_key(&mut resources, *input)?;
            }

            WindowEvent::MouseInput { button, state, .. } => {
                state_machine.on_mouse(&mut resources, button, state)?;
            }

            WindowEvent::DroppedFile(ref path) => {
                state_machine.on_file_dropped(&mut resources, path)?;
            }

            _ => {}
        },
        _ => {}
    }
    Ok(())
}
