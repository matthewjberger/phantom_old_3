use phantom_dependencies::winit::window::Window;

pub struct Resources<'a> {
    pub window: &'a mut Window,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
