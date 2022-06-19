# Game States

Games often have separate maps, levels, areas, and more to wander through. It would not make sense to have all of the assets loaded in memory all of the time, because it would easily consume too much memory and cause unplayable framerates or worse, crashes!

A common way of determining which assets to load and what to display on screen is through the use of `GameStates`. Using these in a stack effectively gives us a [state machine](https://developer.mozilla.org/en-US/docs/Glossary/State_machine) that composes our `GameStates`.

The state on top of the stack is what will be shown in the application. Imagine you are playing a game and you press the pause button. With a state machine, we can simply push another state onto the stack (such as a `GamePaused` state). When we unpause, the state can be popped off the stack and gameplay will resume. This opens the doors to splash screens, loading screens, pause menus, and much more. This flexibility is valuable when trying to manage resources efficiently and write modular code for your game.

## Designing a State Machine

To begin, let's define a trait to represent state in our games. For the `Result` type, we will use `anyhow::Result` because these will be implemented by the application rather than our library.

Declare the `state` module in `crates/phantom_app/lib.rs`.

```rust,noplaypen
...
mod state;

pub use self::{ ..., state::* };
```

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

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn start(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            return Ok(());
        }
        self.running = true;
        self.active_state_mut()?.on_start(resources)
    }

    pub fn handle_event(&mut self, resources: &mut Resources, event: &Event<()>) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_event(resources, &event)?;
        self.transition(transition, resources)
    }

    pub fn update(&mut self, resources: &mut Resources) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.update(resources)?;
        self.transition(transition, resources)
    }

    pub fn on_file_dropped(&mut self, resources: &mut Resources, path: &PathBuf) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_file_dropped(resources, path)?;
        self.transition(transition, resources)
    }

    pub fn on_mouse(
        &mut self,
        resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self
            .active_state_mut()?
            .on_mouse(resources, button, button_state)?;
        self.transition(transition, resources)
    }

    pub fn on_key(&mut self, resources: &mut Resources, input: KeyboardInput) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_key(resources, input)?;
        self.transition(transition, resources)
    }

    pub fn on_event(&mut self, resources: &mut Resources, event: &Event<()>) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        let transition = self.active_state_mut()?.on_event(resources, event)?;
        self.transition(transition, resources)
    }

    fn transition(&mut self, request: Transition, resources: &mut Resources) -> Result<()> {
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
            .context("Tried to access state in state machine with no states present!")
    }

    fn switch(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        if let Some(mut state) = self.states.pop() {
            state.on_stop(resources)?;
        }
        self.states.push(state);
        self.active_state_mut()?.on_start(resources)
    }

    fn push(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        if let Ok(state) = self.active_state_mut() {
            state.on_pause(resources)?;
        }
        self.states.push(state);
        self.active_state_mut()?.on_start(resources)
    }

    fn pop(&mut self, resources: &mut Resources) -> Result<()> {
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

    pub fn stop(&mut self, resources: &mut Resources) -> Result<()> {
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
```

## Using the State Machine

To use the state machine, we'll want to modify our `crates/phantom_app/src/app.rs`.

We should now create a state_machine and pass it to our `run_loop` function.

```rust,noplaypen

pub fn run(initial_state: impl State + 'static, ...) {
    ...
    let mut state_machine = StateMachine::new(initial_state);
    ...
    event_loop.run(move |event, _, control_flow| {
        ...
        if let Err(error) = run_loop(&mut state_machine, &event, control_flow, resources) {
            ...
        }
    });
}

fn run_loop(
    state_machine: &mut StateMachine,
    ...,
) {
    ...
}
```

This allows us to use the `state_machine` in our event handlers!

```rust,noplaypen

fn run_loop(
    ...
) -> Result<()> {
    if !state_machine.is_running() {
        state_machine.start(&mut resources)?;
    }

    state_machine.handle_event(&mut resources, event)?;

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
                ...
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
```
