# Vulkan Instance

The first wrapper needed to create our Vulkan context will be the `Instance`. This is a wrapper around the `ash::Instance` type.

```rust,noplaypen
pub struct Instance {
    pub handle: ash::Instance,
}
```

The constructor on this wrapper is as follows.

```rust,noplaypen
pub fn new(entry: &ash::Entry, extensions: &[*const i8], layers: &[*const i8]) -> Result<Self> {
    let application_create_info = Self::application_create_info()?;
    Self::check_layers_supported(entry, &layers)?;

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&application_create_info)
        .enabled_extension_names(extensions)
        .enabled_layer_names(layers);

    let handle = unsafe { entry.create_instance(&instance_create_info, None) }?;
    Ok(Self { handle })
}
```
