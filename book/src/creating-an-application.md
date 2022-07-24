# Creating an Application

To create various applications and games with our engine, we will need to handle tasks such as creating a window, tracking input from various sources (keyboard, controllers, etc), and keeping track of our application state. These tasks and more will all be encapsulated within our `phantom_app` crate!

## Application Configuration

We'll start off by creating a data structure representing our application's initial configuration. This will allow consumers of the crate to customize features such as window size, the title, the icon, etc.

In our `crates/phantom_app/lib.rs` we'll replace the contents of the file with the following:

```rust,noplaypen
mod app;

pub use self::app::*;
```

Now we will create a file named `crates/phantom_app/app.rs` and begin building our application config.

```rust,noplaypen
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
            title: "Phantom Application".to_string(),
            icon: None,
        }
    }
}
```

## Application Dependencies

Creating a window requires different boilerplate on each platform. Thankfully an excellent open source library for handling window creation named [winit](https://github.com/rust-windowing/winit) exists.

Let's add that to our `phantom_dependencies` project:

```bash
cargo add winit -p phantom_dependencies
```

And let's not forget to re-export it in `crates/phantom_dependencies/lib.rs`!

```rust,noplaypen
pub use winit;
```

## Application Resources

Let's create another module to store resources necessary for the application to run.

### Declaring the Resources Module

In our `crates/phantom_app/lib.rs` we'll declare and export the `resources` module.

```rust,noplaypen
...
mod resources;

pub use self::{
    // ...
    resources::*,
};
```

### Storing Application Resources

Create the file `phantom_app/src/resources.rs` with the following contents:

```rust,noplaypen
use phantom_dependencies::winit::window::Window;

pub struct Resources<'a> {
    pub window: &'a mut Window,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
```

## Application Creation

Now we can do something exciting and get a window visible on screen!

First, run the following command to allow macros from `thiserror` to work for `phantom_app`:

```bash
cargo add thiserror -p phantom_app
```

Now, let's add the following code to `phantom_app/src/app.rs`.

```rust,noplaypen
use crate::Resources;
use phantom_dependencies::{
    env_logger,
    image::{self, io::Reader},
    log,
    thiserror::Error,
    winit::{
        self,
        dpi::PhysicalSize,
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Icon, WindowBuilder},
    },
};

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to create a window!")]
    CreateWindow(#[source] winit::error::OsError),
}

type Result<T, E = ApplicationError> = std::result::Result<T, E>;

// ...

pub fn run(config: AppConfig) -> Result<()> {
    env_logger::init();
    log::info!("Phantom app started");

    let event_loop = EventLoop::new();
    let mut window_builder = WindowBuilder::new()
        .with_title(config.title.to_string())
        .with_inner_size(PhysicalSize::new(config.width, config.height));

    // TODO: Load the window icon

    let mut window = window_builder
        .build(&event_loop)
        .map_err(ApplicationError::CreateWindow)?;

    event_loop.run(move |event, _, control_flow| {
        let resources = Resources {
            window: &mut window,
        };
        if let Err(error) = run_loop(&event, control_flow, resources) {
            log::error!("Application error: {}", error);
        }
    });
}

fn run_loop(event: &Event<()>, control_flow: &mut ControlFlow, resources: Resources) -> Result<()> {
    match event {
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
            }

            _ => {}
        },
        _ => {}
    }
    Ok(())
}

```

### Loading the Window Icon

To load the window icon, we'll need a library for loading images. We'll use the [image](https://crates.io/crates/image) crate.

```bash
cargo add image -p phantom_dependencies
```

Then re-export it in `phantom_dependencies/lib.rs`.

```rust,noplaypen
pub use image;
```

We can now add errors for our icon loading code:

```rust,noplaypen
    #[error("Failed to create icon file!")]
    CreateIcon(#[source] winit::window::BadIcon),

    ...

    #[error("Failed to decode icon file at path: {1}")]
    DecodeIconFile(#[source] image::ImageError, String),

    #[error("Failed to open icon file at path: {1}")]
    OpenIconFile(#[source] io::Error, String),

```

Now we can replace our `TODO` with the following code!

```rust,noplaypen
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
```

### Instantiating the Editor

Now that we've written the necessary window creation code in our library, we can setup our `editor` application.

Replace the contents of `apps/editor/src/main.rs` with the following code.

```rust,noplaypen
use phantom::{
    app::{run, AppConfig},
    dependencies::anyhow::Result,
};

#[derive(Default)]
pub struct Editor;

fn main() -> Result<()> {
    Ok(run(AppConfig {
        icon: Some("assets/icon/phantom.png".to_string()),
        ..Default::default()
    })?)
}
```

Run the application again with `cargo run -r --bin editor` and you should see a blank window with our phantom logo!

### Viewing the console logs

To view the console logs, set the `RUST_LOG` environment variable to `debug`.


#### Mac/Linux:

> RUST_LOG="debug"

#### Windows (powershell):

> $env:RUST_LOG="debug"

