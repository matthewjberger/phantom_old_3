# Vulkan Backend

Now that the `Render` trait exists, we will need to create a backend that implements it.

## Render Backend Feature Flags

To allow compiling a specific backend, we will use [feature flags](https://doc.rust-lang.org/cargo/reference/features.html). For the purpose of this book, we will only be implementing the `Vulkan` backend so it will be a default feature.

```toml
# crates/obsidian_render/Cargo.toml
[features]
default = ["vulkan"]
vulkan = [] 
```

## Vulkan Render Backend

### Setting Up the Backend

Create a new module named `vulkan.rs` and a folder for its modules:

```bash
touch crates/obsidian_render/src/vulkan.rs
mkdir crates/obsidian_render/src/vulkan
```

Update the `crates/obsidian_render/src/lib.rs` to list the new module:

```rust,noplaypen
#[cfg(feature = "vulkan")]
mod vulkan;
```

### Creating the Vulkan Render Module

Create a file for the Vulkan specific render module:

```bash
touch crates/obsidian_render/src/vulkan/render.rs
```

Declare it as a module, and expose the `VulkanRenderBackend` to the crate:

```rust,noplaypen
pub(crate) use self::render::VulkanRenderBackend;

mod render;
```

Declare the `VulkanRenderBackend` as a plain struct that implements the `Render` trait:

```rust,noplaypen
// crates/obsidian_render/src/vulkan/render.rs
use crate::Render;
use anyhow::Result;
use raw_window_handle::HasRawWindowHandle;
use log::info;

pub(crate) struct VulkanRenderBackend;

impl Render for VulkanRenderBackend {
    fn render(
        &mut self,
        _dimensions: &[u32; 2],
    ) -> Result<()> {
        Ok(())
    }
}

impl VulkanRenderBackend {
    pub fn new(
        _window_handle: &impl HasRawWindowHandle,
        _dimensions: &[u32; 2],
    ) -> Result<Self> {
        info!("Created Vulkan render backend");
        Ok(Self{})
    }
} 
```

## Instantiating Graphics Backends

We can now write an associated method for the `Render` trait to provide a [trait object](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) (some type implementing the `Render` trait) by specifying the desired backend.

```rust,noplaypen
// creates/obsidian_render/src/render.rs
#[cfg(feature = "vulkan")]
use crate::vulkan::VulkanRenderBackend;

impl dyn Render {
    pub fn create_backend(
        backend: &Backend,
        window_handle: &impl HasRawWindowHandle,
        dimensions: &[u32; 2],
    ) -> Result<impl Render> {
        match backend {
            Backend::Vulkan => VulkanRenderBackend::new(window_handle, dimensions),
        }
    }
}
```

