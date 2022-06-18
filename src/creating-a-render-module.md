# Creating a Render Module

To keep the rendering code separated from the rest of the codebase, we will create a new module and expose the renderer via a `Render` interface. The `Render` interface is not intended to be a low level common abstraction over various graphics API's, but rather a high level abstraction meant to render the 3D world we will be creating later in this book.

> There is an excellent cross-platform graphics and compute abstraction library in the rust ecosystem named [gfx-rs](https://github.com/gfx-rs/gfx).

## Adding the Render Crate

To start, we can create a crate for handling graphics.

```bash
cargo new --lib crates/obsidian_render
```

To expose it as part of the `obsidian` library, we will want to add this library to the `obsidian/Cargo.toml` [dependencies] section.

```toml
[dependencies]
...
obsidian_render = { path = "crates/obsidian_render" }
```

and expose it as a library module.

```rust,noplaypen
// src/lib.rs

...

pub mod render {
    pub use obsidian_render::*;
}
```

### Dependencies

Add dependencies for error handling, logging, and a new dependency named [raw-window-handle](https://github.com/rust-windowing/raw-window-handle) to crates/obsidian_render/Cargo.toml`:

```toml
anyhow = "1.0.34"
log = "0.4.11"
raw-window-handle = "0.3.3"
```

> `raw-window-handle` is a library that abstracts platform specific window handles. The `winit` library uses the abstraction from this library to provide a window handle via the `HasRawWindowHandle` trait implementation on the `winit::window::Window` type.
