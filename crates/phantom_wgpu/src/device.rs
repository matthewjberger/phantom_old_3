use super::{gui::GuiRender, world::WorldRender};
use phantom_config::Config;
use phantom_gui::GuiFrame;
use phantom_render_traits::GpuDevice;
use phantom_world::{Viewport, World};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;
use wgpu::{
    self, Backends, Device, Queue, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError,
    TextureFormat, TextureViewDescriptor,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to get the current surface texture!")]
    GetSurfaceTexture(#[source] SurfaceError),

    #[error("No suitable GPU adapters found on the system!")]
    NoSuitableGpuAdapters,

    #[error("Failed to find a support swapchain format!")]
    NoSupportedSwapchainFormat,

    #[error("Failed to request a device!")]
    RequestDevice(#[source] RequestDeviceError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct WgpuRenderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub gui: GuiRender,
    pub depth_texture_view: wgpu::TextureView,
    pub world_render: Option<WorldRender>,
}

impl GpuDevice for WgpuRenderer {
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
        self.depth_texture_view =
            create_depth_texture(&self.config, &self.device, Self::DEPTH_FORMAT);
        Ok(())
    }

    fn render_frame(
        &mut self,
        world: &mut World,
        _config: &Config,
        gui_frame: &mut GuiFrame,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let GuiFrame {
            textures_delta,
            screen_descriptor,
            paint_jobs,
        } = gui_frame;
        self.gui
            .update_textures(&self.device, &self.queue, textures_delta);
        self.gui.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            paint_jobs,
            screen_descriptor,
        );

        let aspect_ratio = self.aspect_ratio();
        if let Some(world_render) = self.world_render.as_mut() {
            world_render.update(&self.queue, aspect_ratio, world);
        }

        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(Error::GetSurfaceTexture)?;

        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        {
            encoder.insert_debug_marker("Render scene");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if let Some(world_render) = self.world_render.as_ref() {
                world_render.render(&mut render_pass, world)?;
            }

            self.gui
                .render(&mut render_pass, paint_jobs, screen_descriptor);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

impl WgpuRenderer {
    const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window_handle: &W,
        backend: wgpu::Backend,
        viewport: &Viewport,
    ) -> Result<Self> {
        pollster::block_on(WgpuRenderer::new_async(window_handle, backend, viewport))
    }

    async fn new_async<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window_handle: &W,
        backend: wgpu::Backend,
        viewport: &Viewport,
    ) -> Result<Self> {
        let backend: Backends = backend.into();

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(window_handle) };

        let adapter = Self::create_adapter(&instance, &surface, backend).await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let swapchain_format = *surface
            .get_supported_formats(&adapter)
            .first()
            .ok_or(Error::NoSupportedSwapchainFormat)?;

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: viewport.width as _,
            height: viewport.height as _,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let gui = GuiRender::new(&device, config.format, Some(Self::DEPTH_FORMAT), 1);

        let depth_texture_view = create_depth_texture(&config, &device, Self::DEPTH_FORMAT);

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
            .ok_or(Error::NoSuitableGpuAdapters)
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
            .map_err(Error::RequestDevice)
    }
}

fn create_depth_texture(
    config: &SurfaceConfiguration,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
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
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };

    let texture = device.create_texture(&texture_descriptor);

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}