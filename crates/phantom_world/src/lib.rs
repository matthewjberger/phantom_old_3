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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Hidden;

#[derive(Serialize, Deserialize)]
pub struct Name(pub String);
