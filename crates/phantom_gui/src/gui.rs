use phantom_dependencies::{
    egui::{Context as GuiContext, FullOutput},
    egui_winit::State,
    winit::{event::WindowEvent, event_loop::EventLoopWindowTarget, window::Window},
};

pub struct Gui {
    pub state: State,
    pub context: GuiContext,
}

impl Gui {
    pub fn new<T>(window: &Window, event_loop: &EventLoopWindowTarget<T>) -> Self {
        let state = State::new(event_loop);
        let context = GuiContext::default();
        context.set_pixels_per_point(window.scale_factor() as f32);
        Self { state, context }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        let Gui { state, context } = self;
        state.on_event(context, event)
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let gui_input = self.state.take_egui_input(window);
        self.context.begin_frame(gui_input);
    }

    pub fn end_frame(&mut self) -> FullOutput {
        self.context.end_frame()
    }
}