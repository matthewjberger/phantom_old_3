use phantom::{
    app::{
        run, AppConfig, ApplicationError, MouseOrbit, Resources, State, StateResult, Transition,
    },
    dependencies::{
        anyhow::anyhow,
        egui::{self, global_dark_light_mode_switch, menu},
        log,
        rfd::FileDialog,
        winit::event::{ElementState, KeyboardInput, MouseButton},
    },
};

#[derive(Default)]
pub struct Editor {
    camera: MouseOrbit,
}

impl State for Editor {
    fn label(&self) -> String {
        "Phantom Editor - Main".to_string()
    }

    fn update(&mut self, resources: &mut Resources) -> StateResult<Transition> {
        if resources.world.active_camera_is_main()? {
            let camera_entity = resources.world.active_camera()?;
            self.camera.update(resources, camera_entity)?;
        }
        Ok(Transition::None)
    }

    fn update_gui(&mut self, resources: &mut Resources) -> StateResult<Transition> {
        let ctx = &resources.gui.context.clone();

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                menu::bar(ui, |ui| {
                    global_dark_light_mode_switch(ui);
                    ui.menu_button("File", |ui| {
                        if ui.button("Create New Map").clicked() {
                            // TODO: Create map
                        }

                        if ui.button("Load Map").clicked() {
                            let path = FileDialog::new()
                                .add_filter("Dragonglass Asset", &["dga"])
                                .set_directory("/")
                                .pick_file();
                            if let Some(path) = path {
                                resources.load_map(&path).unwrap();
                            }
                            ui.close_menu();
                        }

                        if ui.button("Import asset (gltf/glb)").clicked() {
                            let path = FileDialog::new()
                                .add_filter("GLTF Asset", &["glb", "gltf"])
                                .set_directory("/")
                                .pick_file();
                            if let Some(path) = path {
                                resources.load_gltf_asset(&path).unwrap();
                            }
                            ui.close_menu();
                        }

                        if ui.button("Load HDR Image").clicked() {
                            // TODO: Load map
                            // resources.load_map(path)
                        }

                        if ui.button("Save").clicked() {
                            let path = FileDialog::new()
                                .add_filter("Dragonglass Asset", &["dga"])
                                .set_directory("/")
                                .save_file();
                            if let Some(path) = path {
                                resources.world.save(&path).expect("Failed to save world!");
                            }
                            ui.close_menu();
                        }

                        if ui.button("Quit").clicked() {
                            resources.system.exit_requested = true;
                        }
                    });
                });
            });

        egui::SidePanel::left("scene_explorer")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Scene Explorer");
                ui.allocate_space(ui.available_size());
            });

        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.allocate_space(ui.available_size());
            });

        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Assets");
                ui.allocate_space(ui.available_size());
            });

        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        path: &std::path::Path,
    ) -> StateResult<Transition> {
        log::info!(
            "File dropped: {}",
            path.as_os_str()
                .to_str()
                .ok_or_else(|| anyhow!("Failed to get file path to dropped file!"))?
        );
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> StateResult<Transition> {
        log::trace!("Mouse event: {:#?} {:#?}", button, button_state);
        Ok(Transition::None)
    }

    fn on_key(
        &mut self,
        _resources: &mut Resources,
        input: KeyboardInput,
    ) -> StateResult<Transition> {
        log::trace!("Key event received: {:#?}", input);
        Ok(Transition::None)
    }
}

fn main() -> Result<(), ApplicationError> {
    run(
        Editor::default(),
        AppConfig {
            icon: Some("assets/icons/phantom.png".to_string()),
            ..Default::default()
        },
    )
}
