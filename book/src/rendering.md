# Rendering

For rendering, we'll use [wgpu](https://github.com/gfx-rs/wgpu) which is a safe and portable GPU abstraction in rust that implements the WebGPU API. Without this library, we would have write our own GPU abstraction and create separate backends for that abstraction. Supporting as many different platforms as possible for the sake of this engine provides flexibility. We could render on `Android` with [Vulkan](https://www.vulkan.org/), `Windows` with [DirectX](https://developer.nvidia.com/directx), `IOS` and `MacOS` with [Metal](https://developer.apple.com/metal/), or maybe even the web using [webgl](https://www.khronos.org/webgl/) (and eventually [webgpu](https://www.w3.org/TR/webgpu/)!).