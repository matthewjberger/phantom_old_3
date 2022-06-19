use phantom::{
    app::{run, AppConfig, Resources, State, Transition},
    dependencies::{
        anyhow::Result,
        log,
        winit::event::{ElementState, Event, KeyboardInput, MouseButton},
    },
};

#[derive(Default)]
pub struct Editor;

impl State for Editor {
    fn on_start(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Starting the Phantom editor");
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Stopping the Phantom editor");
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Editor paused");
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Editor unpaused");
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        path: &std::path::PathBuf,
    ) -> Result<Transition> {
        log::info!(
            "File dropped: {}",
            path.as_os_str().to_str().expect("Failed to convert path!")
        );
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> Result<Transition> {
        log::trace!("Mouse event: {:#?} {:#?}", button, button_state,);
        Ok(Transition::None)
    }

    fn on_key(&mut self, _resources: &mut Resources, input: KeyboardInput) -> Result<Transition> {
        log::trace!("Key event received: {:#?}", input);
        Ok(Transition::None)
    }

    fn on_event(&mut self, _resources: &mut Resources, event: &Event<()>) -> Result<Transition> {
        log::trace!("Event received: {:#?}", event);
        Ok(Transition::None)
    }
}

fn main() -> Result<()> {
    Ok(run(
        Editor::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            ..Default::default()
        },
    )?)
}
