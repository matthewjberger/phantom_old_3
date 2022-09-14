use super::{gui::GuiRender, world::WorldRender};
use crate::{Backend, Renderer};
use phantom_config::Config;
use phantom_dependencies::{
    egui::ClippedPrimitive,
    egui_wgpu::renderer::ScreenDescriptor,
    log, pollster,
    raw_window_handle::HasRawWindowHandle,
    thiserror::Error,
    wgpu::{
        self, Backend as WgpuBackend, Backends, Device, Queue, RequestDeviceError, Surface,
        SurfaceConfiguration, SurfaceError, TextureViewDescriptor,
    },
};
use phantom_gui::GuiFrameResources;
use phantom_world::{Viewport, World};

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

pub(crate) struct WgpuRenderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub gui: GuiRender,
    pub depth_texture_view: wgpu::TextureView,
    pub world_render: Option<WorldRender>,
}

impl Renderer for WgpuRenderer {
    fn load_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        self.world_render = Some(WorldRender::new(&self.device, self.config.format, world));
        Ok(())
    }

    fn resize(&mut self, dimensions: [u32; 2]) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "Resizing renderer surface to: ({}, {})",
            dimensions[0],
            dimensions[1]
        );
        if dimensions[0] == 0 || dimensions[1] == 0 {
            return Ok(());
        }
        self.config.width = dimensions[0];
        self.config.height = dimensions[1];
        self.surface.configure(&self.device, &self.config);
        self.depth_texture_view = create_depth_texture(&self.config, &self.device);
        Ok(())
    }

    fn update(
        &mut self,
        world: &mut World,
        _config: &Config,
        gui_frame_resources: &mut GuiFrameResources,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let GuiFrameResources {
            textures_delta,
            screen_descriptor,
            paint_jobs,
        } = gui_frame_resources;
        self.gui
            .update_textures(&self.device, &self.queue, textures_delta);
        self.gui
            .update_buffers(&self.device, &self.queue, screen_descriptor, paint_jobs);

        let aspect_ratio = self.aspect_ratio();
        if let Some(world_render) = self.world_render.as_mut() {
            world_render.update(&self.queue, aspect_ratio, world);
        }

        Ok(())
    }

    fn render_frame(
        &mut self,
        world: &mut World,
        _config: &Config,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        // {
        //     encoder.insert_debug_marker("Render scene");
        //     let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.1,
        //                     g: 0.2,
        //                     b: 0.3,
        //                     a: 1.0,
        //                 }),
        //                 store: true,
        //             },
        //         })],
        //         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        //             view: &self.depth_texture_view,
        //             depth_ops: Some(wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(1.0),
        //                 store: true,
        //             }),
        //             stencil_ops: None,
        //         }),
        //     });
        //     if let Some(world_render) = self.world_render.as_ref() {
        //         world_render.render(&mut renderpass, world);
        //     }
        // }

        self.gui
            .execute(&mut encoder, &view, paint_jobs, screen_descriptor, None);

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

impl WgpuRenderer {
    pub fn new(
        window_handle: &impl HasRawWindowHandle,
        backend: &Backend,
        viewport: &Viewport,
    ) -> Result<Self> {
        pollster::block_on(WgpuRenderer::new_async(window_handle, backend, viewport))
    }

    async fn new_async(
        window_handle: &impl HasRawWindowHandle,
        backend: &Backend,
        viewport: &Viewport,
    ) -> Result<Self> {
        let backend: Backends = map_backend(backend)?.into();

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(window_handle) };

        let adapter = Self::create_adapter(&instance, &surface, backend).await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let swapchain_format = *surface
            .get_supported_formats(&adapter)
            .first()
            .ok_or(RendererError::NoSupportedSwapchainFormat)?;

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: viewport.width as _,
            height: viewport.height as _,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let gui = GuiRender::new(&device, config.format, 1);

        let depth_texture_view = create_depth_texture(&config, &device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            gui,
            depth_texture_view,
            world_render: None,
        })
    }

    fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / std::cmp::max(1, self.config.height) as f32
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
        backend: Backends,
    ) -> Result<wgpu::Adapter> {
        wgpu::util::initialize_adapter_from_env_or_default(instance, backend, Some(surface))
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

fn map_backend(backend: &Backend) -> Result<WgpuBackend> {
    let backend = match backend {
        Backend::Dx11 => WgpuBackend::Dx11,
        Backend::Dx12 => WgpuBackend::Dx12,
        Backend::Metal => WgpuBackend::Metal,
        Backend::Vulkan => WgpuBackend::Vulkan,
    };
    Ok(backend)
}

fn create_depth_texture(config: &SurfaceConfiguration, device: &wgpu::Device) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };

    let texture = device.create_texture(&texture_descriptor);

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
