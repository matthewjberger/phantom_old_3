use super::world::WorldRender;
use crate::Renderer;
use phantom_dependencies::{
    anyhow::Result,
    egui::ClippedPrimitive,
    egui_glow::{self, glow, Painter},
    egui_wgpu::renderer::ScreenDescriptor,
    gl,
    glutin::{window::Window, ContextWrapper, PossiblyCurrent},
};
use phantom_world::{Viewport, World};
use std::sync::Arc;

pub struct OpenGlRenderer {
    world_render: Option<WorldRender>,
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

        Ok(Self {
            world_render: None,
            viewport: *viewport,
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
        _world: &mut World,
        gui_frame_resources: &mut phantom_gui::GuiFrameResources,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let textures_delta = gui_frame_resources.textures_delta;
        for (id, image_delta) in textures_delta.set.iter() {
            self.gui_painter.set_texture(*id, image_delta);
        }
        Ok(())
    }

    fn render_frame(
        &mut self,
        world: &mut World,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
        context: &ContextWrapper<PossiblyCurrent, Window>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }

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

//     fn set_viewport(&mut self, viewport: Viewport) {
//         unsafe {
//             gl::Viewport(
//                 viewport.x as _,
//                 viewport.y as _,
//                 viewport.width as _,
//                 viewport.height as _,
//             );
//         }
//         self.viewport = viewport;
//     }