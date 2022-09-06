use phantom_dependencies::serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Config {
    pub graphics: Graphics,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Graphics {
    pub post_processing: PostProcessing,
    pub debug_grid_active: bool,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct PostProcessing {
    pub film_grain: FilmGrain,
    pub chromatic_aberration: ChromaticAberration,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct ChromaticAberration {
    pub strength: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct FilmGrain {
    pub strength: f32,
}