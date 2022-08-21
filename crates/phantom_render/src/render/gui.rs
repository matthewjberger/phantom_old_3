use phantom_dependencies::{
    egui::{self, ClippedPrimitive, TexturesDelta},
    egui_wgpu::renderer::{RenderPass, ScreenDescriptor},
    wgpu::{self, Device, Queue},
};

pub struct GuiRender {
    pub gui_renderpass: RenderPass,
}

impl GuiRender {
    pub fn new(device: &Device, output_format: wgpu::TextureFormat, msaa_samples: u32) -> Self {
        let gui_renderpass = RenderPass::new(device, output_format, msaa_samples);
        Self { gui_renderpass }
    }

    pub fn update_textures(
        &mut self,
        device: &Device,
        queue: &Queue,
        textures_delta: &TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.gui_renderpass
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &textures_delta.free {
            self.gui_renderpass.free_texture(id);
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        paint_jobs: &[ClippedPrimitive],
    ) {
        self.gui_renderpass
            .update_buffers(device, queue, paint_jobs, screen_descriptor);
    }

    pub fn execute<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        color_attachment: &wgpu::TextureView,
        paint_jobs: &'a [egui::epaint::ClippedPrimitive],
        screen_descriptor: &'a ScreenDescriptor,
        clear_color: Option<wgpu::Color>,
    ) {
        self.gui_renderpass.execute(
            encoder,
            color_attachment,
            paint_jobs,
            screen_descriptor,
            clear_color,
        );
    }
}
