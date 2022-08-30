use super::{graphics::Graphics, grid::GridShader, world::WorldRender};
use crate::Renderer;
use phantom_dependencies::{
    anyhow::Result,
    egui::ClippedPrimitive,
    egui_glow::{self, glow, Painter},
    egui_wgpu::renderer::ScreenDescriptor,
    gl,
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
    nalgebra_glm as glm,
};
use phantom_world::{Viewport, World};
use std::sync::Arc;

pub struct OpenGlRenderer {
    world_render: Option<WorldRender>,
    grid: GridShader,
    viewport: Viewport,
    glow_context: Arc<glow::Context>,
    gui_painter: Painter,
}

impl OpenGlRenderer {
    pub fn new(
        context: &ContextWrapper<PossiblyCurrent, Window>,
        viewport: &Viewport,
    ) -> Result<Self> {
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        let glow_context = unsafe {
            glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol))
        };
        let glow_context = Arc::new(glow_context);
        let gui_painter = egui_glow::Painter::new(glow_context.clone(), None, "").unwrap();

        let grid = GridShader::new()?;

        Ok(Self {
            world_render: None,
            viewport: *viewport,
            grid,
            glow_context,
            gui_painter,
        })
    }
}

impl Renderer for OpenGlRenderer {
    fn sync_world(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        self.world_render = Some(WorldRender::new(world)?);
        Ok(())
    }

    fn resize(
        &mut self,
        dimensions: [u32; 2],
        context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: dimensions[0] as _,
            height: dimensions[1] as _,
        };
        unsafe {
            gl::Viewport(
                self.viewport.x as _,
                self.viewport.y as _,
                self.viewport.width as _,
                self.viewport.height as _,
            );
        }
        context.resize(dimensions.into());
        Ok(())
    }

    fn update(
        &mut self,
        world: &mut World,
        gui_frame_resources: &mut phantom_gui::GuiFrameResources,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let textures_delta = gui_frame_resources.textures_delta;
        for (id, image_delta) in textures_delta.set.iter() {
            self.gui_painter.set_texture(*id, image_delta);
        }

        let (projection, view) = world
            .active_camera_matrices(self.viewport.aspect_ratio())
            .unwrap();
        let camera_entity = world.active_camera().unwrap();
        let camera_transform = world.entity_global_transform(camera_entity).unwrap();
        self.grid
            .update(view, projection, camera_transform.translation);

        Ok(())
    }

    fn render_frame(
        &mut self,
        world: &mut World,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
        context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Graphics::clear_buffers();
        Graphics::clear_color(&glm::vec3(0.3, 0.3, 0.3));

        self.grid.render();

        if let Some(world_render) = self.world_render.as_ref() {
            world_render.render(world, self.viewport.aspect_ratio())?;
        }

        self.gui_painter.paint_primitives(
            screen_descriptor.size_in_pixels,
            screen_descriptor.pixels_per_point,
            paint_jobs,
        );

        context.swap_buffers()?;

        Ok(())
    }
}
