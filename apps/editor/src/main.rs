use phantom::{
    app::{run, AppConfig, ApplicationError, Resources, State, StateResult, Transition},
    dependencies::{
        anyhow::anyhow,
        gilrs::Event as GilrsEvent,
        log,
        winit::event::{ElementState, Event, KeyboardInput, MouseButton},
    },
};

#[derive(Default)]
pub struct Editor;

impl State for Editor {
    fn label(&self) -> String {
        "Phantom Editor - Main".to_string()
    }

    fn on_start(&mut self, _resources: &mut Resources) -> StateResult<()> {
        log::info!("Starting the Phantom editor");
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> StateResult<()> {
        log::info!("Stopping the Phantom editor");
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> StateResult<()> {
        log::info!("Editor paused");
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> StateResult<()> {
        log::info!("Editor unpaused");
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_gamepad_event(
        &mut self,
        _resources: &mut Resources,
        event: GilrsEvent,
    ) -> StateResult<Transition> {
        let GilrsEvent { id, time, event } = event;
        log::trace!("{:?} New gamepad event from {}: {:?}", time, id, event);
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        path: &std::path::Path,
    ) -> StateResult<Transition> {
        log::info!(
            "File dropped: {}",
            path.as_os_str()
                .to_str()
                .ok_or_else(|| anyhow!("Failed to get file path to dropped file!"))?
        );
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> StateResult<Transition> {
        log::trace!("Mouse event: {:#?} {:#?}", button, button_state);
        Ok(Transition::None)
    }

    fn on_key(
        &mut self,
        _resources: &mut Resources,
        input: KeyboardInput,
    ) -> StateResult<Transition> {
        log::trace!("Key event received: {:#?}", input);
        Ok(Transition::None)
    }

    fn on_event(
        &mut self,
        _resources: &mut Resources,
        _event: &Event<()>,
    ) -> StateResult<Transition> {
        Ok(Transition::None)
    }
}

fn main() -> Result<(), ApplicationError> {
    run(
        Editor::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            ..Default::default()
        },
    )
}
