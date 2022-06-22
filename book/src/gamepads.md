# Gamepads

To handle gamepads, we will use the [gilrs](https://gitlab.com/gilrs-project/gilrs) library. This library abstracts platform specific APIs to provide unified interfaces for working with gamepads and supports a wide variety of controllers.

To integrate this library, let's first add `gilrs` as a dependency.

```bash
cargo add gilrs -p nalgebra_glm
```

Then, export it in `crates/phantom_dependencies/src/lib.rs`:

```bash
pub use gilrs as glm;
```

Let's extend this gamepad support to our game states by adding it to our `Resources` bundle in `crates/phantom_app/src/resources.rs`.

```rust,noplaypen
...

use phantom_dependencies::{
    gilrs::Gilrs,
    ...
};

pub struct Resources<'a> {
    ...
    pub gilrs: &'a mut Gilrs,
}
```

Next, we'll have to add an instance of `Gilrs` to the `Resources` that we declare in the application boilerplate in `crates/phantom_app/src/app.rs`.

```rust,noplaypen
...

use phantom_dependencies::{
    anyhow::{self, anyhow},
    env_logger,
    gilrs::Gilrs,
    ...
}

...

pub fn run(...) {
    ...

    let mut gilrs = Gilrs::new().map_err(|_err| anyhow!("Failed to setup gamepad library!"))?;

    ...

    event_loop.run(move |event, _, control_flow| {
       let resources = Resources {
            ...
            gilrs: &mut gilrs,
        };  
        ...
    })
}

pub fn run_loop(...) {
    ...

    if let Some(event) = resources.gilrs.next_event() {
        state_machine.on_gamepad_event(&mut resources, event)?;
    }

    ...

}
...
```

At this point, you'll notice that we haven't implemented anything in our state machine to handle gamepad events. Let's add a method to our `State` trait in `crates/phantom_app/src/state.rs`!

```rust,noplaypen
...
use phantom_dependencies::{
    ...
    gilrs::Event as GilrsEvent,
};
...

trait State {
    ...

    fn on_gamepad_event(
        &mut self,
        _resources: &mut Resources,
        _event: GilrsEvent,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }
}
```

With this declared, we can now command our state machine to forward gamepad events to the game states.

Add the following method to our `StateMachine` in `crates/phantom_app/src/state.rs`.

```rust,noplaypen
pub fn on_gamepad_event(&mut self, resources: &mut Resources, event: GilrsEvent) -> Result<()> {
    if !self.running {
        return Ok(());
    }
    let transition = self
        .active_state_mut()?
        .on_gamepad_event(resources, event)?;
    self.transition(transition, resources)
}
```

Let's make use of this new event handler by adding the following code to our editor at `apps/editor/src/main.rs`.

```rust,noplaypen
use phantom::{
    ...
    dependencies::{
        anyhow::{Context, Result},
        gilrs::Event as GilrsEvent,
        ...
    },
};

...

impl State for Editor {
    ...
    fn on_gamepad_event(
        &mut self,
        _resources: &mut Resources,
        event: GilrsEvent,
    ) -> Result<Transition> {
        let GilrsEvent { id, time, event } = event;
        log::trace!("{:?} New gamepad event from {}: {:?}", time, id, event);
        Ok(Transition::None)
    }
}
```

Now, with a controller hooked up you'll be able to interact with your application!

> Set $env:RUST_LOG="debug" on windows or RUST_LOG="debug" on mac/linux to view the logs
