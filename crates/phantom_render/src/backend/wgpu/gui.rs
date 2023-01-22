use egui::{self, TexturesDelta};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use wgpu::{self, Device, Queue};

pub struct GuiRender {
    pub renderer: Renderer,
}

impl GuiRender {
    pub fn new(
        device: &Device,
        output_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
    ) -> Self {
        Self {
            renderer: Renderer::new(device, output_format, depth_format, msaa_samples),
        }
    }

    pub fn update_textures(
        &mut self,
        device: &Device,
        queue: &Queue,
        textures_delta: &TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &textures_delta.free {
            self.renderer.free_texture(id);
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        paint_jobs: &[egui::epaint::ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
    ) {
        self.renderer
            .update_buffers(device, queue, encoder, paint_jobs, screen_descriptor);
    }

    pub fn render<'rp>(
        &'rp self,
        render_pass: &mut wgpu::RenderPass<'rp>,
        paint_jobs: &[egui::epaint::ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
    ) {
        self.renderer
            .render(render_pass, paint_jobs, screen_descriptor);
    }
}
