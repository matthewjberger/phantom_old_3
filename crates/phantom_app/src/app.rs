use crate::{Input, Resources, State, StateMachine, System};
use phantom_dependencies::{
    egui::FullOutput,
    egui_wgpu::renderer::ScreenDescriptor,
    env_logger,
    gilrs::{self, Gilrs},
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
use phantom_gui::Gui;
use phantom_render::{Renderer, RendererError, Viewport};
use phantom_world::{World, WorldError};
use std::io;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to create icon file!")]
    CreateIcon(#[source] winit::window::BadIcon),

    #[error("Failed to create a window!")]
    CreateWindow(#[source] winit::error::OsError),

    #[error("Failed to create world!")]
    CreateWorld(#[source] WorldError),

    #[error("Failed to create the renderer!")]
    CreateRenderer(#[source] RendererError),

    #[error("Failed to decode icon file at path: {1}")]
    DecodeIconFile(#[source] image::ImageError, String),

    #[error("Failed to handle an event in the state machine!")]
    HandleEvent(#[source] Box<dyn std::error::Error>),

    #[error("Failed to initialize the gamepad input library!")]
    InitializeGamepadLibrary(#[source] gilrs::Error),

    #[error("Failed to open icon file at path: {1}")]
    OpenIconFile(#[source] io::Error, String),

    #[error("Failed to render a frame!")]
    RenderFrame(#[source] RendererError),

    #[error("Failed to start the state machine!")]
    StartStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to stop the state machine!")]
    StopStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to update the renderer!")]
    UpdateRenderer(#[source] RendererError),

    #[error("Failed to update the state machine!")]
    UpdateStateMachine(#[source] Box<dyn std::error::Error>),

    #[error("Failed to to update the gui!")]
    UpdateGui(#[source] Box<dyn std::error::Error>),
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
        let image = Reader::open(icon_path)
            .map_err(|error| ApplicationError::OpenIconFile(error, icon_path.to_string()))?
            .decode()
            .map_err(|error| ApplicationError::DecodeIconFile(error, icon_path.to_string()))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height)
            .map_err(ApplicationError::CreateIcon)?;
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let mut window = window_builder
        .build(&event_loop)
        .map_err(ApplicationError::CreateWindow)?;

    if config.is_fullscreen {
        window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
    }

    let mut state_machine = StateMachine::new(initial_state);

    let physical_size = window.inner_size();
    let window_dimensions = [physical_size.width, physical_size.height];

    let mut gilrs = Gilrs::new().map_err(ApplicationError::InitializeGamepadLibrary)?;

    let mut input = Input::default();
    let mut system = System::new(window_dimensions);

    let mut renderer = Renderer::new(
        &window,
        &Viewport {
            width: config.width,
            height: config.height,
            ..Default::default()
        },
    )
    .map_err(ApplicationError::CreateRenderer)?;

    let mut gui = Gui::new(&window, &event_loop);

    let mut world = World::new().map_err(ApplicationError::CreateWorld)?;

    event_loop.run(move |event, _, control_flow| {
        let resources = Resources {
            world: &mut world,
            window: &mut window,
            gui: &mut gui,
            gilrs: &mut gilrs,
            input: &mut input,
            system: &mut system,
        };
        if let Err(error) = run_loop(
            &mut state_machine,
            &event,
            control_flow,
            &mut renderer,
            resources,
        ) {
            log::error!("Application error: {}", error);
        }
    });
}

fn run_loop(
    state_machine: &mut StateMachine,
    event: &Event<()>,
    control_flow: &mut ControlFlow,
    renderer: &mut Renderer,
    mut resources: Resources,
) -> Result<()> {
    if resources.system.exit_requested {
        *control_flow = ControlFlow::Exit;
    }

    let gui_captured_event = match event {
        Event::WindowEvent { event, window_id } => {
            if *window_id == resources.window.id() {
                resources.gui.handle_window_event(event)
            } else {
                false
            }
        }
        _ => false,
    };

    if !gui_captured_event {
        resources.system.handle_event(event);
        resources
            .input
            .handle_event(event, resources.system.window_center());
    }

    if !state_machine.is_running() {
        state_machine
            .start(&mut resources)
            .map_err(ApplicationError::StartStateMachine)?;
    }

    state_machine
        .handle_event(&mut resources, event)
        .map_err(ApplicationError::HandleEvent)?;

    if let Some(event) = resources.gilrs.next_event() {
        state_machine
            .on_gamepad_event(&mut resources, event)
            .map_err(ApplicationError::HandleEvent)?;
    }

    match event {
        Event::MainEventsCleared => {
            resources.gui.begin_frame(resources.window);
            state_machine
                .update_gui(&mut resources)
                .map_err(ApplicationError::UpdateGui)?;
            let output = resources.gui.end_frame();

            let FullOutput {
                textures_delta,
                shapes,
                ..
            } = output;
            let paint_jobs = resources.gui.context.tessellate(shapes);
            let window_size = resources.window.inner_size();
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [window_size.width, window_size.height],
                pixels_per_point: resources.window.scale_factor() as f32,
            };

            state_machine
                .update(&mut resources)
                .map_err(ApplicationError::UpdateStateMachine)?;

            renderer
                .update(&textures_delta, &screen_descriptor, &paint_jobs)
                .map_err(ApplicationError::UpdateRenderer)?;
            renderer
                .render_frame(&paint_jobs, &screen_descriptor)
                .map_err(ApplicationError::RenderFrame)?;
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

                state_machine
                    .on_key(&mut resources, *input)
                    .map_err(ApplicationError::HandleEvent)?;
            }

            WindowEvent::MouseInput { button, state, .. } => {
                state_machine
                    .on_mouse(&mut resources, button, state)
                    .map_err(ApplicationError::HandleEvent)?;
            }

            WindowEvent::DroppedFile(ref path) => {
                state_machine
                    .on_file_dropped(&mut resources, path)
                    .map_err(ApplicationError::HandleEvent)?;
            }

            WindowEvent::Resized(physical_size) => {
                renderer.resize([physical_size.width, physical_size.height]);
                state_machine
                    .on_resize(&mut resources, physical_size)
                    .map_err(ApplicationError::HandleEvent)?;
            }

            _ => {}
        },

        Event::LoopDestroyed => {
            state_machine
                .stop(&mut resources)
                .map_err(ApplicationError::StopStateMachine)?;
        }

        _ => {}
    }
    Ok(())
}
