# Creating the Rendering Library

To keep the rendering code separated from the rest of the codebase, we will create a new library called `obsidian_render`.

## Adding the Render Crate

To start, we can create a crate for handling graphics.

```bash
cargo new --lib crates/obsidian_render
```

Then we can link `obsidian` against the `obsidian_render` library, by listing it as a dependency in `obsidian/Cargo.toml`

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

Add dependencies for error handling, logging, and a new dependency named [raw-window-handle](https://github.com/rust-windowing/raw-window-handle) to `crates/obsidian_render/Cargo.toml`.

```toml
anyhow = "1.0.34"
log = "0.4.11"
raw-window-handle = "0.3.3"
```

> `raw-window-handle` is a library that abstracts platform specific window handles. The `winit` library uses the abstraction from this library to provide a window handle via the `HasRawWindowHandle` trait implementation on the `Window` type.
