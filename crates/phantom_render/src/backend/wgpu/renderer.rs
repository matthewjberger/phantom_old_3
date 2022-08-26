use super::{
    gui::GuiRender,
    world::{render::WorldRender, texture::Texture},
};
use crate::Renderer;
use phantom_dependencies::{
    anyhow,
    egui::ClippedPrimitive,
    egui_wgpu::renderer::ScreenDescriptor,
    glutin::{ContextWrapper, PossiblyCurrent},
    log, pollster,
    raw_window_handle::HasRawWindowHandle,
    thiserror::Error,
    wgpu::{
        self, Device, Queue, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError,
        TextureViewDescriptor,
    },
    winit::window::Window,
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

    #[error("Failed to load world!")]
    LoadWorld(#[source] anyhow::Error),

    #[error("Failed to update world!")]
    UpdateWorld(#[source] anyhow::Error),
}

type Result<T, E = RendererError> = std::result::Result<T, E>;

pub struct WgpuRenderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub gui: GuiRender,
    pub world_render: WorldRender,
    pub depth_texture: Texture,
}

impl Renderer for WgpuRenderer {
    fn sync_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        self.world_render
            .load(&self.device, &self.queue, world)
            .map_err(RendererError::LoadWorld)?;
        Ok(())
    }

    fn resize(
        &mut self,
        dimensions: [u32; 2],
        _context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        self.depth_texture = Texture::create_depth_texture(
            &self.device,
            dimensions[0],
            dimensions[1],
            "Depth Texture",
        );
        Ok(())
    }

    fn update(
        &mut self,
        world: &mut World,
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
        self.world_render
            .update(&self.queue, world, self.aspect_ratio())
            .map_err(RendererError::UpdateWorld)?;
        Ok(())
    }

    fn render_frame(
        &mut self,
        world: &mut World,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
        _context: &ContextWrapper<PossiblyCurrent, Window>,
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

        {
            encoder.insert_debug_marker("Render scene");
            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.world_render.render(&mut renderpass, world).unwrap();
        }

        self.gui
            .execute(&mut encoder, &view, paint_jobs, screen_descriptor, None);

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

impl WgpuRenderer {
    pub fn new(window_handle: &impl HasRawWindowHandle, viewport: &Viewport) -> Result<Self> {
        pollster::block_on(WgpuRenderer::new_async(window_handle, viewport))
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
            width: viewport.width as _,
            height: viewport.height as _,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let gui = GuiRender::new(&device, config.format, 1);

        let depth_texture = Texture::create_depth_texture(
            &device,
            viewport.width as _,
            viewport.height as _,
            "Depth Texture",
        );

        let world_render = WorldRender::new(&device, config.format).unwrap();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            gui,
            depth_texture,
            world_render,
        })
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / std::cmp::max(1, self.config.height) as f32
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
