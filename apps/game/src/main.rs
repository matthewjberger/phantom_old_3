use game::Game;
use phantom::app::{run, AppConfig, ApplicationError};

mod game;

fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    run(
        Game::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            title: "Phantom Viewer".to_string(),
            is_fullscreen: true,
            ..Default::default()
        },
    )
}
