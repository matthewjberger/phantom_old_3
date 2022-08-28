use phantom::{
    app::{MouseOrbit, Resources, State, StateResult, Transition},
    dependencies::{
        anyhow::anyhow,
        egui::{self, global_dark_light_mode_switch, menu, SelectableLabel, Ui},
        log,
        petgraph::{graph::NodeIndex, Direction::Outgoing},
        rfd::FileDialog,
        winit::event::{ElementState, KeyboardInput, MouseButton},
    },
    world::{Ecs, Entity, Name, SceneGraph},
};

use crate::commands::{CommandList, LoadGltfAssetCommand};

#[derive(Default)]
pub struct Editor {
    camera: MouseOrbit,
    selected_entities: Vec<Entity>,
    commands: CommandList,
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
                                self.commands
                                    .queue_command(Box::new(LoadGltfAssetCommand(path)))
                                    .unwrap();
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
                            resources.close_map().unwrap();
                        }

                        if ui.button("Quit").clicked() {
                            resources.system.exit_requested = true;
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        ui.add_enabled_ui(self.commands.has_undo_commands(), |ui| {
                            if ui.button("Undo").clicked() {
                                self.commands.undo(resources).unwrap();
                            }
                        });

                        ui.add_enabled_ui(self.commands.has_redo_commands(), |ui| {
                            if ui.button("Redo").clicked() {
                                self.commands.redo(resources).unwrap();
                            }
                        });
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

                ui.label(format!("Scene Name: {}", &resources.world.scene.name));

                ui.heading("Entities");

                let scene = &mut resources.world.scene;
                let ecs = &mut resources.world.ecs;
                for (graph_index, graph) in scene.graphs.iter_mut().enumerate() {
                    for node_index in graph.root_node_indices().unwrap() {
                        self.print_node(ecs, graph, graph_index, node_index, ui);
                    }
                }

                ui.allocate_space(ui.available_size());
            });
    }

    fn print_node(
        &mut self,
        ecs: &mut Ecs,
        graph: &mut SceneGraph,
        graph_index: usize,
        entity_index: NodeIndex,
        ui: &mut Ui,
    ) {
        let entity = graph[entity_index];
        let selected = self.selected_entities.contains(&entity);

        let context_menu = |ui: &mut Ui| {
            if ui.button("Delete...").clicked() {
                ui.close_menu();
            }

            if ui.button("Add Child...").clicked() {
                // UI TODO: Allow adding child entities
                ui.close_menu();
            }
        };

        let header = {
            let mut entry = ecs.entry(entity).expect("Failed to find entity!");
            match entry.get_component_mut::<Name>() {
                Ok(name) => name.0.to_string(),
                Err(_) => {
                    entry.add_component(Name(format!("{:?}", entity)));
                    entry.get_component_mut::<Name>().unwrap().0.to_string()
                }
            }
        };

        let response = if graph.has_children(entity_index) {
            egui::CollapsingHeader::new(header.to_string())
                .show(ui, |ui| {
                    let mut neighbors = graph.neighbors(entity_index, Outgoing);
                    while let Some(child) = neighbors.next_node(&graph.0) {
                        self.print_node(ecs, graph, graph_index, child, ui);
                    }
                })
                .header_response
                .context_menu(context_menu)
        } else {
            ui.add(SelectableLabel::new(selected, header))
                .context_menu(context_menu)
        };

        if response.clicked() {
            if !self.selected_entities.contains(&entity) || self.selected_entities.len() > 0 {
                self.selected_entities = vec![entity];
            }
        }

        if response.double_clicked() {
            // TODO: Allow renaming entity
        }
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
        self.commands.execute_pending_commands(resources)?;
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
