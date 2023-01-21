use egui::{self, ClippedPrimitive, TexturesDelta};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use wgpu::{self, Device, Queue};

pub struct GuiRender {
    pub gui_renderer: Renderer,
}

impl GuiRender {
    pub fn new(
        device: &Device,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
    ) -> Self {
        Self {
            gui_renderer: Renderer::new(device, color_format, depth_format, msaa_samples),
        }
    }

    pub fn update_textures(
        &mut self,
        device: &Device,
        queue: &Queue,
        textures_delta: &TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.gui_renderer
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &textures_delta.free {
            self.gui_renderer.free_texture(id);
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        paint_jobs: &[ClippedPrimitive],
    ) {
        // TODO: Update the wgpu egui renderer
        // self.gui_renderer
        //     .update_buffers(device, queue, paint_jobs, screen_descriptor);
    }

    pub fn execute<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        color_attachment: &wgpu::TextureView,
        paint_jobs: &'a [egui::epaint::ClippedPrimitive],
        screen_descriptor: &'a ScreenDescriptor,
        clear_color: Option<wgpu::Color>,
    ) {
        // TODO: Update the wgpu egui renderer
        // self.gui_renderer.execute(
        //     encoder,
        //     color_attachment,
        //     paint_jobs,
        //     screen_descriptor,
        //     clear_color,
        // );
    }
}
