use phantom::{
    app::{run, AppConfig},
    dependencies::anyhow::Result,
};

#[derive(Default)]
pub struct Editor;

fn main() -> Result<()> {
    Ok(run(AppConfig {
        icon: Some("assets/icons/phantom.png".to_string()),
        ..Default::default()
    })?)
}
