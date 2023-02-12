use crate::Resources;
use gilrs::Event as GilrsEvent;
use phantom_window::winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, MouseButton},
};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateMachineError {
    #[error("Failed to get the current surface texture!")]
    NoStatesPresent,
}

type Result<T, E = StateMachineError> = std::result::Result<T, E>;
pub type StateResult<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub struct EmptyState {}
impl State for EmptyState {}

pub trait State {
    fn label(&self) -> String {
        "Unlabeled Game State".to_string()
    }

    fn on_start(&mut self, _resources: &mut Resources) -> StateResult<()> {
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> StateResult<()> {
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> StateResult<()> {
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> StateResult<()> {
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn update_gui(&mut self, _resources: &mut Resources) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_gamepad_event(
        &mut self,
        _resources: &mut Resources,
        _event: GilrsEvent,
    ) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        _path: &Path,
    ) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_resize(
        &mut self,
        _resources: &mut Resources,
        _physical_size: &PhysicalSize<u32>,
    ) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        _button: &MouseButton,
        _button_state: &ElementState,
    ) -> StateResult<Transition> {
        Ok(Transition::None)
    }

    fn on_key(
        &mut self,
        _resources: &mut Resources,
        _input: KeyboardInput,
    ) -> StateResult<Transition> {
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

pub enum Transition {
    None,
    Pop,
    Push(Box<dyn State>),
    Switch(Box<dyn State>),
    Quit,
}

pub struct StateMachine {
    running: bool,
    states: Vec<Box<dyn State>>,
}

impl StateMachine {
    pub fn new(initial_state: impl State + 'static) -> Self {
        Self {
            running: false,
            states: vec![Box::new(initial_state)],
        }
    }

    pub fn active_state_label(&self) -> Option<String> {
        if !self.running {
            return None;
        }
        self.states.last().map(|state| state.label())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn start(&mut self, resources: &mut Resources) -> StateResult<()> {
        if self.running {
            return Ok(());
        }
        self.running = true;
        self.active_state_mut()?.on_start(resources)
    }

    pub fn handle_event(
        &mut self,
        resources: &mut Resources,
        event: &Event<()>,
    ) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_event(resources, event)?;
        self.transition(transition, resources)
    }

    pub fn update(&mut self, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.update(resources)?;
        self.transition(transition, resources)
    }

    pub fn update_gui(&mut self, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.update_gui(resources)?;
        self.transition(transition, resources)
    }

    pub fn on_gamepad_event(
        &mut self,
        resources: &mut Resources,
        event: GilrsEvent,
    ) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self
            .active_state_mut()?
            .on_gamepad_event(resources, event)?;
        self.transition(transition, resources)
    }

    pub fn on_file_dropped(&mut self, resources: &mut Resources, path: &Path) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_file_dropped(resources, path)?;
        self.transition(transition, resources)
    }

    pub fn on_resize(
        &mut self,
        resources: &mut Resources,
        physical_size: &PhysicalSize<u32>,
    ) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self
            .active_state_mut()?
            .on_resize(resources, physical_size)?;
        self.transition(transition, resources)
    }

    pub fn on_mouse(
        &mut self,
        resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self
            .active_state_mut()?
            .on_mouse(resources, button, button_state)?;
        self.transition(transition, resources)
    }

    pub fn on_key(&mut self, resources: &mut Resources, input: KeyboardInput) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_key(resources, input)?;
        self.transition(transition, resources)
    }

    pub fn on_event(&mut self, resources: &mut Resources, event: &Event<()>) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_event(resources, event)?;
        self.transition(transition, resources)
    }

    fn transition(&mut self, request: Transition, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        match request {
            Transition::None => Ok(()),
            Transition::Pop => self.pop(resources),
            Transition::Push(state) => self.push(state, resources),
            Transition::Switch(state) => self.switch(state, resources),
            Transition::Quit => self.stop(resources),
        }
    }

    fn active_state_mut(&mut self) -> Result<&mut Box<(dyn State + 'static)>> {
        self.states
            .last_mut()
            .ok_or(StateMachineError::NoStatesPresent)
    }

    fn switch(&mut self, state: Box<dyn State>, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        if let Some(mut state) = self.states.pop() {
            state.on_stop(resources)?;
        }
        self.states.push(state);
        self.active_state_mut()?.on_start(resources)
    }

    fn push(&mut self, state: Box<dyn State>, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        if let Ok(state) = self.active_state_mut() {
            state.on_pause(resources)?;
        }
        self.states.push(state);
        self.active_state_mut()?.on_start(resources)
    }

    fn pop(&mut self, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }

        if let Some(mut state) = self.states.pop() {
            state.on_stop(resources)?;
        }

        if let Some(state) = self.states.last_mut() {
            state.on_resume(resources)
        } else {
            self.running = false;
            Ok(())
        }
    }

    pub fn stop(&mut self, resources: &mut Resources) -> StateResult<()> {
        if !self.running {
            return Ok(());
        }
        while let Some(mut state) = self.states.pop() {
            state.on_stop(resources)?;
        }
        self.running = false;
        Ok(())
    }
}
