use phantom::{
    app::{MouseOrbit, Resources, State, StateResult, Transition},
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

impl Editor {
    fn top_panel(&mut self, resources: &mut Resources) {
        let ctx = &resources.gui.context.clone();
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                menu::bar(ui, |ui| {
                    global_dark_light_mode_switch(ui);
                    ui.menu_button("File", |ui| {
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

                        if ui.button("Save Map").clicked() {
                            let path = FileDialog::new()
                                .add_filter("Dragonglass Asset", &["dga"])
                                .set_directory("/")
                                .save_file();
                            if let Some(path) = path {
                                resources.world.save(&path).expect("Failed to save world!");
                            }
                            ui.close_menu();
                        }

                        if ui.button("Close map").clicked() {
                            // TODO: If unsaved, ask before closing
                            resources.reset_world().unwrap();
                        }

                        if ui.button("Quit").clicked() {
                            resources.system.exit_requested = true;
                        }
                    });
                });
            });
    }

    fn left_panel(&mut self, resources: &mut Resources) {
        let ctx = &resources.gui.context.clone();
        egui::SidePanel::left("scene_explorer")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Scene Explorer");
                ui.allocate_space(ui.available_size());
            });
    }

    fn right_panel(&mut self, resources: &mut Resources) {
        let ctx = &resources.gui.context.clone();
        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.allocate_space(ui.available_size());
            });
    }

    fn bottom_panel(&mut self, resources: &mut Resources) {
        let ctx = &resources.gui.context.clone();
        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Assets");
                ui.allocate_space(ui.available_size());
            });
    }
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
        self.top_panel(resources);
        self.left_panel(resources);
        self.right_panel(resources);
        self.bottom_panel(resources);
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
