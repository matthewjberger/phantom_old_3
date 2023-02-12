use editor::Editor;
use phantom::{
    app::{run, AppConfig, ApplicationError},
    window::WindowConfig,
};

mod commands;
mod editor;

fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    run(
        Editor::default(),
        AppConfig {
            window: WindowConfig {
                icon: Some("assets/icons/phantom.png".to_string()),
                title: "Phantom Editor".to_string(),
                ..Default::default()
            },
        },
    )
}
