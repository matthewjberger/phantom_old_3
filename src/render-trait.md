# Render Trait

To prevent coupling any one specific backend to the rest of the application, the render library can expose the renderer via a `Render` interface and consumers of the library can request a specific backend via a `Backend` enum. The `Render` interface is not intended to be a low level common abstraction over various graphics API's, but rather a high level abstraction meant to render the 3D world we will be creating later in this book.

> There is an excellent cross-platform graphics and compute abstraction library in the rust ecosystem named [gfx-rs](https://github.com/gfx-rs/gfx).

## The Render Module

Create a new module named `render.rs`

```bash
touch crates/obsidian_render/src/render.rs
```

Update the `crates/obsidian_render/src/lib.rs` to list the new module:

```rust,noplaypen
pub mod render;
pub use crate::render::{Backend, Render};
```


We can list our graphics backends with an enum:

```rust,noplaypen
pub enum Backend {
    Vulkan,
}
```

The `Render` trait can be written as:

```rust,noplaypen
pub trait Render {
    fn render(
        &mut self,
        dimensions: &[u32; 2],
    ) -> Result<()>;
}
```

> The `render` call will eventually be given a parameter containing a description of our `World` to render. The `World` implementation will come in a later chapter.
