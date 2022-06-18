# Game States

Games often have separate maps, levels, areas, and more to wander through. It would not make sense to have all of the assets loaded in memory all of the time, because it would easily consume too much memory and cause unplayable framerates or worse, crashes!

A common way of determining which assets to load and what to display on screen is through the use of `GameState`s. Using these in a stack effectively gives us a [state machine](https://developer.mozilla.org/en-US/docs/Glossary/State_machine) that composes our `GameState`s.

The state on top of the stack is what will be shown in the application. Imagine you are playing a game and you press the pause button. With a state machine, we can simply push another state onto the stack (such as a `GamePaused` state). When we unpause, the state can be popped off the stack and gameplay will resume. This opens the doors to splash screens, loading screens, pause menus, and much more. This flexibility is valuable when trying to manage resources efficiently and write modular code for your game.

## Designing a State Machine

To begin, let's define a trait to represent state in our games. For the `Result` type, we will use `anyhow::Result` because these will be implemented by the application rather than our library.

Create a file named `crates/phantom_app/state.rs` with the following contents.

```rust,noplaypen
use crate::Resources;
use phantom_dependencies::{
    anyhow::{Context, Result},
    winit::event::{ElementState, Event, KeyboardInput, MouseButton},
};
use std::path::PathBuf;

pub trait State {
    fn on_start(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn update_gui(&mut self, _resources: &mut Resources) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        _path: &PathBuf,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        _button: &MouseButton,
        _button_state: &ElementState,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_key(&mut self, _resources: &mut Resources, _input: KeyboardInput) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_event(&mut self, _resources: &mut Resources, _event: &Event<()>) -> Result<Transition> {
        Ok(Transition::None)
    }
}
```

Now, let's define a type for representing transitions between game states.

```rust,noplaypen
pub enum Transition {
    None,
    Pop,
    Push(Box<dyn State>),
    Switch(Box<dyn State>),
    Quit,
}
```

With these traits defined, we are ready to define our `StateMachine`.

Note that the states are not public, as they should be accessed internally by the state machine itself. All methods on the state resolve to a `Transition` that is used to determine whether or not transition the state machine. These transitions should happen automatically from the user's perspective!

```rust,noplaypen
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

    fn current_state(&mut self) -> Result<&mut Box<(dyn State + 'static)>> {
        self.states
            .last_mut()
            .context("Tried to access state in state machine with no states present!")
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn start(&mut self, resources: &mut Resources) -> Result<()> {
        if !self.running {
            let state = self.current_state()?;
            state.on_start(resources)?;
            self.running = true;
        }
        Ok(())
    }

    pub fn handle_event(&mut self, resources: &mut Resources, event: &Event<()>) -> Result<()> {
        if self.running {
            let transition = match self.states.last_mut() {
                Some(state) => state.on_event(resources, &event)?,
                None => Transition::None,
            };
            self.transition(transition, resources)?;
        }
        Ok(())
    }

    pub fn update(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            let transition = match self.states.last_mut() {
                Some(state) => state.update(resources)?,
                None => Transition::None,
            };
            self.transition(transition, resources)?;
        }
        Ok(())
    }

    pub fn transition(&mut self, request: Transition, resources: &mut Resources) -> Result<()> {
        if self.running {
            match request {
                Transition::None => (),
                Transition::Pop => self.pop(resources)?,
                Transition::Push(state) => self.push(state, resources)?,
                Transition::Switch(state) => self.switch(state, resources)?,
                Transition::Quit => self.stop(resources)?,
            }
        }
        Ok(())
    }

    fn switch(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            self.states.push(state);
            let new_state = self.current_state()?;
            new_state.on_start(resources)?;
        }
        Ok(())
    }

    fn push(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Ok(state) = self.current_state() {
                state.on_pause(resources)?;
            }
            self.states.push(state);
            let new_state = self.current_state()?;
            new_state.on_start(resources)?;
        }
        Ok(())
    }

    fn pop(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            if let Some(state) = self.states.last_mut() {
                state.on_resume(resources)?;
            } else {
                self.running = false;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            while let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            self.running = false;
        }
        Ok(())
    }
}
```
