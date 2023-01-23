pub mod app {
    pub use phantom_app::*;
}

pub mod audio {
    pub use phantom_audio::*;
}

pub mod gui {
    pub use phantom_gui::*;
}

pub mod render {
    pub use phantom_render::*;
}

pub mod world {
    pub use legion;
    pub use petgraph;
    pub use phantom_world::*;
    pub use rapier3d;
}
