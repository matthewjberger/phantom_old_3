# Tracking Input

Using the events we receive from Winit, we have a way of responding to events as they happen,
but we don't necessarily know the state of a given key, the mouse, the window, etc. What we want is
for that information to be tracked and made available to gamestates, so they can make
informed decisions using up-to-date information.

## Necessary Dependencies

First, let's add a math library to simplify some of the calculations.

> This will be heavily used when we design our world crate!

```bash
cargo add anyhow -p nalgebra_glm
```

Then, export it in `crates/phantom_dependencies/src/lib.rs`:

```bash
pub use nalgebra_glm as glm;
```

## Adding an Input Resource

Let's add a file at `crates/phantom_app/src/resources/input.rs`.

```rust,noplaypen
use phantom_dependencies::{
    glm,
    winit::{
        dpi::PhysicalPosition,
        event::{
            ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
            WindowEvent,
        },
    },
};
use std::collections::HashMap;

pub type KeyMap = HashMap<VirtualKeyCode, ElementState>;

pub struct Input {
    pub keystates: KeyMap,
    pub mouse: Mouse,
    pub allowed: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keystates: KeyMap::default(),
            mouse: Mouse::default(),
            allowed: true,
        }
    }
}

impl Input {
    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keystates.contains_key(&keycode) && self.keystates[&keycode] == ElementState::Pressed
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>, window_center: glm::Vec2) {
        if !self.allowed {
            return;
        }

        if let Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state,
                            ..
                        },
                    ..
                },
            ..
        } = *event
        {
            *self.keystates.entry(keycode).or_insert(state) = state;
        }

        self.mouse.handle_event(event, window_center);
    }
}

#[derive(Default)]
pub struct Mouse {
    pub is_left_clicked: bool,
    pub is_right_clicked: bool,
    pub position: glm::Vec2,
    pub position_delta: glm::Vec2,
    pub offset_from_center: glm::Vec2,
    pub wheel_delta: glm::Vec2,
    pub moved: bool,
    pub scrolled: bool,
}

impl Mouse {
    pub fn handle_event<T>(&mut self, event: &Event<T>, window_center: glm::Vec2) {
        match event {
            Event::NewEvents { .. } => self.new_events(),
            Event::WindowEvent { event, .. } => match *event {
                WindowEvent::MouseInput { button, state, .. } => self.mouse_input(button, state),
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_moved(position, window_center)
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(h_lines, v_lines),
                    ..
                } => self.mouse_wheel(h_lines, v_lines),
                _ => {}
            },
            _ => {}
        }
    }

    fn new_events(&mut self) {
        if !self.scrolled {
            self.wheel_delta = glm::vec2(0.0, 0.0);
        }
        self.scrolled = false;

        if !self.moved {
            self.position_delta = glm::vec2(0.0, 0.0);
        }
        self.moved = false;
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>, window_center: glm::Vec2) {
        let last_position = self.position;
        let current_position = glm::vec2(position.x as _, position.y as _);
        self.position = current_position;
        self.position_delta = current_position - last_position;
        self.offset_from_center = window_center - glm::vec2(position.x as _, position.y as _);
        self.moved = true;
    }

    fn mouse_wheel(&mut self, h_lines: f32, v_lines: f32) {
        self.wheel_delta = glm::vec2(h_lines, v_lines);
        self.scrolled = true;
    }

    fn mouse_input(&mut self, button: MouseButton, state: ElementState) {
        let clicked = state == ElementState::Pressed;
        match button {
            MouseButton::Left => self.is_left_clicked = clicked,
            MouseButton::Right => self.is_right_clicked = clicked,
            _ => {}
        }
    }
}
```

This lets us easily track keystates and mouse information!

## Adding a System Resource

Let's add a file at `crates/phantom_app/src/resources/system.rs`.

```rust,noplaypen
use phantom_dependencies::{
    glm,
    winit::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
    },
};
use std::{cmp, time::Instant};

pub struct System {
    pub window_dimensions: [u32; 2],
    pub delta_time: f64,
    pub last_frame: Instant,
    pub exit_requested: bool,
}

impl System {
    pub fn new(window_dimensions: [u32; 2]) -> Self {
        Self {
            last_frame: Instant::now(),
            window_dimensions,
            delta_time: 0.01,
            exit_requested: false,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        let width = self.window_dimensions[0];
        let height = cmp::max(self.window_dimensions[1], 0);
        width as f32 / height as f32
    }

    pub fn window_center(&self) -> glm::Vec2 {
        glm::vec2(
            self.window_dimensions[0] as f32 / 2.0,
            self.window_dimensions[1] as f32 / 2.0,
        )
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>) {
        match event {
            Event::NewEvents { .. } => {
                self.delta_time = (Instant::now().duration_since(self.last_frame).as_micros()
                    as f64)
                    / 1_000_000_f64;
                self.last_frame = Instant::now();
            }
            Event::WindowEvent { event, .. } => match *event {
                WindowEvent::CloseRequested => self.exit_requested = true,
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    self.window_dimensions = [width, height];
                }
                _ => {}
            },
            _ => {}
        }
    }
}

```

Now, we won't have to pollute our main game loop just to keep track of window dimensions and frame latency.

## Adding Resources

Now, we can modify our `crates/phantom_app/src/resources.rs` to include our new resources.

```rust,noplaypen
mod input;
mod system;

pub use self::{input::*, system::*};

use phantom_dependencies::winit::window::Window;

pub struct Resources<'a> {
    pub window: &'a mut Window,
    pub input: &'a mut Input,
    pub system: &'a mut System,
}

...
```

## Instantiating Resources

Finally, we can instantiate our resources in our `crates/phantom_app/src/app.rs`.

```rust,noplaypen
use crate::{Input, Resources, State, StateMachine, System};

pub fn run(...) {
    ...

    let physical_size = window.inner_size();
    let window_dimensions = [physical_size.width, physical_size.height];

    let mut input = Input::default();
    let mut system = System::new(window_dimensions);

    ...

    event_loop.run(move |event, _, control_flow| {
       let resources = Resources {
            ...
            input: &mut input,
            system: &mut system,
        };  
        ...
    })
}
...
```
