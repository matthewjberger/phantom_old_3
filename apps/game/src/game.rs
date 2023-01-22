use legion::EntityStore;
use nalgebra_glm as glm;
use phantom::{
    app::{MouseLook, Resources, State, StateResult, Transition},
    world::{Camera, Entity, PerspectiveCamera, Projection, Transform},
};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Default)]
pub struct Game {
    player: Option<Entity>,
    camera: MouseLook,
}

impl State for Game {
    fn label(&self) -> String {
        "Phantom Game".to_string()
    }

    fn on_start(&mut self, resources: &mut Resources) -> StateResult<()> {
        resources.world.add_default_light()?;
        resources.load_gltf("assets/models/blocklevel.glb").unwrap();

        {
            let position = glm::vec3(0.0, 1.0, 0.0);
            let transform = Transform {
                translation: position,
                ..Default::default()
            };
            let player_entity = resources.world.ecs.push((transform,));
            resources
                .world
                .scene
                .default_scenegraph_mut()?
                .add_root_node(player_entity);
            activate_first_person(resources, player_entity)?;
            self.player = Some(player_entity);
        }

        Ok(())
    }

    fn update(&mut self, resources: &mut Resources) -> StateResult<Transition> {
        if let Some(player) = self.player.as_ref() {
            update_player(resources, *player)?;
            self.camera.update(resources, *player)?;
        }
        Ok(Transition::None)
    }

    fn on_key(
        &mut self,
        resources: &mut Resources,
        input: KeyboardInput,
    ) -> StateResult<Transition> {
        if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
            (input.virtual_keycode, input.state)
        {
            resources.system.exit_requested = true;
        }
        Ok(Transition::None)
    }
}

fn update_player(resources: &mut Resources, entity: Entity) -> StateResult<()> {
    let speed = 2.0 * resources.system.delta_time as f32;
    {
        let mut entry = resources.world.ecs.entry_mut(entity)?;
        let transform = entry.get_component_mut::<Transform>()?;
        let mut translation = glm::vec3(0.0, 0.0, 0.0);

        if resources.input.is_key_pressed(VirtualKeyCode::W) {
            translation = speed * transform.forward();
        }

        if resources.input.is_key_pressed(VirtualKeyCode::A) {
            translation = -speed * transform.right();
        }

        if resources.input.is_key_pressed(VirtualKeyCode::S) {
            translation = -speed * transform.forward();
        }

        if resources.input.is_key_pressed(VirtualKeyCode::D) {
            translation = speed * transform.right();
        }

        transform.translation += translation;
    }
    Ok(())
}

fn activate_first_person(resources: &mut Resources, entity: Entity) -> StateResult<()> {
    // Disable active camera
    let camera_entity = resources.world.active_camera()?;
    resources
        .world
        .ecs
        .entry_mut(camera_entity)?
        .get_component_mut::<Camera>()?
        .enabled = false;

    resources
        .world
        .ecs
        .entry(entity)
        .unwrap()
        .add_component(Camera {
            name: "Player Camera".to_string(),
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 90_f32.to_radians(),
                z_far: Some(1000.0),
                z_near: 0.001,
            }),
            enabled: true,
        });

    Ok(())
}
