mod animation;
mod camera;
mod gltf;
mod physics;
mod registry;
mod scenegraph;
mod texture;
mod transform;
mod world;

pub use self::{
    animation::*, camera::*, gltf::*, physics::*, registry::*, scenegraph::*, texture::*,
    transform::*, world::*,
};
use phantom_dependencies::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Hidden;

#[derive(Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Name(pub String);
