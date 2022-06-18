# Using the Renderer

Now we can instantiate a renderer in our `obsidian_app` library, before moving on to filling out the Vulkan graphics backend.

List the `obsidian_render` library as a dependency in `crates/obsidian_app/Cargo.toml`:

```toml
[dependencies]
...
obsidian_render = { path = "../obsidian_render" }
```

Now we can instantiate the renderer by modifying `crates/obsidian_app/src/app.rs`.

Import the library types:

```rust,noplaypen
use obsidian_render::{Render, Backend};
```

Add a `renderer` property to the `Application` struct as a boxed trait object, and instantiate it in a constructor:

```rust,noplaypen
pub struct Application {
    pub renderer: Box<dyn Render>,
}

impl Application {
    pub fn new(window: &Window) -> Result<Self> {
        let logical_size = window.inner_size();
        let window_dimensions = [logical_size.width, logical_size.height];
        let renderer = Box::new(Render::create_backend(
            &Backend::Vulkan,
            window,
            &window_dimensions,
        )?);
        Ok(Self { renderer })
    }
}
```

The `Application` can now be created with the constructor in the `run_application` method:

```rust,noplaypen
// '_window' becomes 'window' here since it is now used
let (event_loop, window) = create_window(&configuration)?;

let mut application = Application::new(&window)?;
```

If you run the program now with `cargo run --release`, the log output should include a message saying that the Vulkan render backend was created.