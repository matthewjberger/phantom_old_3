use anyhow::anyhow;
use phantom::app::{MouseOrbit, Resources, State, StateResult, Transition};

#[derive(Default)]
pub struct Viewer {
    camera: MouseOrbit,
}

impl State for Viewer {
    fn label(&self) -> String {
        "Phantom Viewer".to_string()
    }

    fn update(&mut self, resources: &mut Resources) -> StateResult<Transition> {
        if resources.world.active_camera_is_main()? {
            let camera_entity = resources.world.active_camera()?;
            self.camera.update(resources, camera_entity)?;
        }
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        resources: &mut Resources,
        path: &std::path::Path,
    ) -> StateResult<Transition> {
        log::info!(
            "File dropped: {}",
            path.as_os_str()
                .to_str()
                .ok_or_else(|| anyhow!("Failed to get file path to dropped file!"))?
        );

        resources.world.clear()?;
        resources.world.add_default_light()?;
        resources.load_gltf(path).unwrap();

        Ok(Transition::None)
    }
}
