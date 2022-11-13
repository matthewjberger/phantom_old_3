use phantom::app::{run, AppConfig, ApplicationError};
use viewer::Viewer;

mod viewer;

fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    run(
        Viewer::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            title: "Phantom Viewer".to_string(),
            ..Default::default()
        },
    )
}
