# Vulkan Context

Now we can start using Vulkan! This section will detail a self contained structure for setting up and store the fundamental objects required to run a Vulkan application.

> From this point on in the book, the instructions for setting up and creating the files will not detail each line and step, and will rely on the accompanying source code for reference. All important sections of code will be explained, however!

The structure will be called a Vulkan `Context`. This is not an official Vulkan term, but rather the name for our grouping of Vulkan objects.

```rust,noplaypen
pub struct Context {
    pub allocator: Arc<vk_mem::Allocator>,
    pub device: Arc<Device>,
    pub physical_device: PhysicalDevice,
    pub surface: Option<Surface>,
    pub instance: Instance,
    pub entry: ash::Entry,
}
```

The order the struct fields are declared in determines the order that they are `Drop`ped in. This will become important later, as cleaning up Vulkan resources has to happen in a particular order. Resources must not be in use when they are destructed, so declaring the struct this way enforces the correct order.

In reverse order, the fields are as follows.

* `entry`

    The function loader from the `ash` library.

* `instance`

    A wrapper around a [`vk::Instance`](https://docs.rs/ash/0.31.0/ash/vk/struct.Instance.html), which stores application state because there is no global state in Vulkan.

* `surface`

    A wrapper around a [`ash::extension::khr::Surface`](https://docs.rs/ash/0.31.0/ash/extensions/khr/struct.Surface.html). A surface is required when rendering to a window.

* `physical_device`

    A wrapper around a [`vk::PhysicalDevice`](https://docs.rs/ash/0.31.0/ash/vk/struct.PhysicalDevice.html). On simple systems, this a physical device represents a specific, physical GPU.

* `device`

    A wrapper around a [`vk::Device`](https://docs.rs/ash/0.31.0/ash/vk/struct.Device.html). A logical device represents the application's view of the physical device. Vulkan calls will be made primarily on this object.

* `allocator`

    Vulkan requires the application to handle allocating memory on its own. Thankfully, AMD has released a library that does this task well, called the [Vulkan Memory Allocator](https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator). The rust bindings are provided by [vk-mem-rs](https://github.com/gwihlidal/vk-mem-rs). Using a type from the `vk-mem-rs` library, we can create and store a memory allocator.

## Vulkan Handles

Vulkan objects are constructed via API calls, and return handles. The wrappers we will create instantiate particular Vulkan objects, store their handles, and implement the `Drop` trait to call a `destroy_instance` API call which frees the resource. This intentionally ties the lifetime of the Vulkan object to lifetime of the Rust wrapper type.
