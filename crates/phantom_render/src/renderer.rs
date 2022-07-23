use phantom_dependencies::{
    log, pollster,
    raw_window_handle::HasRawWindowHandle,
    thiserror::Error,
    wgpu::{
        self, Device, Queue, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError,
        TextureViewDescriptor,
    },
};
use std::cmp::max;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to get the current surface texture!")]
    GetSurfaceTexture(#[source] SurfaceError),

    #[error("No suitable GPU adapters found on the system!")]
    NoSuitableGpuAdapters,

    #[error("Failed to find a support swapchain format!")]
    NoSupportedSwapchainFormat,

    #[error("Failed to request a device!")]
    RequestDevice(#[source] RequestDeviceError),
}

type Result<T, E = RendererError> = std::result::Result<T, E>;

#[derive(Default, Copy, Clone)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / max(self.height, 0) as f32
    }
}

pub struct Renderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

impl Renderer {
    pub fn new(window_handle: &impl HasRawWindowHandle, viewport: &Viewport) -> Result<Self> {
        pollster::block_on(Renderer::new_async(window_handle, viewport))
    }

    pub fn update(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn resize(&mut self, dimensions: [u32; 2]) {
        log::info!(
            "Resizing renderer surface to: ({}, {})",
            dimensions[0],
            dimensions[1]
        );
        if dimensions[0] == 0 || dimensions[1] == 0 {
            return;
        }
        self.config.width = dimensions[0];
        self.config.height = dimensions[1];
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render_frame(&mut self) -> Result<()> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(RendererError::GetSurfaceTexture)?;

        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.insert_debug_marker("Render scene");
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / std::cmp::max(1, self.config.height) as f32
    }

    async fn new_async(
        window_handle: &impl HasRawWindowHandle,
        viewport: &Viewport,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(Self::backends());

        let surface = unsafe { instance.create_surface(window_handle) };

        let adapter = Self::create_adapter(&instance, &surface).await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let swapchain_format = *surface
            .get_supported_formats(&adapter)
            .first()
            .ok_or(RendererError::NoSupportedSwapchainFormat)?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: viewport.width,
            height: viewport.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
        })
    }

    fn backends() -> wgpu::Backends {
        wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all)
    }

    fn required_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
        wgpu::Limits::default()
            // Use the texture resolution limits from the adapter
            // to support images the size of the surface
            .using_resolution(adapter.limits())
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Result<wgpu::Adapter> {
        wgpu::util::initialize_adapter_from_env_or_default(
            instance,
            Self::backends(),
            Some(surface),
        )
        .await
        .ok_or(RendererError::NoSuitableGpuAdapters)
    }

    async fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
        log::info!("WGPU Adapter Features: {:#?}", adapter.features());

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: (Self::optional_features() & adapter.features())
                        | Self::required_features(),
                    limits: Self::required_limits(adapter),
                    label: Some("Render Device"),
                },
                None,
            )
            .await
            .map_err(RendererError::RequestDevice)
    }
}
