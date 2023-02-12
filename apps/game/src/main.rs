use game::Game;
use phantom::{
    app::{run, AppConfig, ApplicationError},
    window::WindowConfig,
};

mod game;

fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    run(
        Game::default(),
        AppConfig {
            window: WindowConfig {
                icon: Some("assets/icons/phantom.png".to_string()),
                title: "Phantom Game".to_string(),
                ..Default::default()
            },
        },
    )
}
