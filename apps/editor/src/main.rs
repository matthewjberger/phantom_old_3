use editor::Editor;
use phantom::app::{run, AppConfig, ApplicationError};

mod commands;
mod editor;

fn main() -> Result<(), ApplicationError> {
    run(
        Editor::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            title: "Phantom Editor".to_string(),
            ..Default::default()
        },
    )
}
