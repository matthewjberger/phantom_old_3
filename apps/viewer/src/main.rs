use phantom::{
    app::{run, AppConfig, ApplicationError},
    window::WindowConfig,
};
use viewer::Viewer;

mod viewer;

fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    run(
        Viewer::default(),
        AppConfig {
            window: WindowConfig {
                icon: Some("assets/icons/phantom.png".to_string()),
                title: "Phantom viewer".to_string(),
                ..Default::default()
            },
        },
    )
}
