use editor::Editor;
use phantom::{
    app::{run, AppConfig, ApplicationError},
    render::Backend,
};

mod editor;

fn main() -> Result<(), ApplicationError> {
    run(
        Editor::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            render_backend: Backend::Vulkan,
            ..Default::default()
        },
    )
}
