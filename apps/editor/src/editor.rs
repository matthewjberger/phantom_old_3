use crate::commands::{
    CloseMapCommand, CommandList, ExitCommand, LoadGltfAssetCommand, OpenMapCommand, SaveMapCommand,
};
use anyhow::anyhow;
use phantom::{
    app::{MouseOrbit, Resources, State, StateResult, Transition},
    gui::{
        egui::{self, global_dark_light_mode_switch, menu, LayerId, SelectableLabel, Ui},
        egui_gizmo::{GizmoMode, GizmoOrientation},
        GizmoWidget,
    },
    world::{
        legion::EntityStore,
        nalgebra_glm as glm,
        petgraph::{graph::NodeIndex, Direction::Outgoing},
        Ecs, Entity, EntitySceneGraph, Name, RigidBody, Transform,
    },
};
use rfd::FileDialog;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct Editor {
    camera: MouseOrbit,
    selected_entities: Vec<Entity>,
    commands: CommandList,
    gizmo: GizmoWidget,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            camera: MouseOrbit::default(),
            selected_entities: Vec::new(),
            commands: CommandList::default(),
            gizmo: GizmoWidget::new(),
        }
    }
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
                                .add_filter("Phantom Map", &["pha"])
                                .set_directory("/")
                                .pick_file();
                            if let Some(path) = path {
                                self.commands
                                    .queue_command(Box::new(OpenMapCommand(path)))
                                    .unwrap();
                            }
                            ui.close_menu();
                        }

                        if ui.button("Save Map").clicked() {
                            let path = FileDialog::new()
                                .add_filter("Phantom Map", &["pha"])
                                .set_directory("/")
                                .save_file();
                            if let Some(path) = path {
                                self.commands
                                    .queue_command(Box::new(SaveMapCommand(path)))
                                    .unwrap();
                            }
                            ui.close_menu();
                        }

                        if ui.button("Close map").clicked() {
                            self.commands
                                .queue_command(Box::new(CloseMapCommand {}))
                                .unwrap();
                            self.selected_entities.clear();
                        }

                        if ui.button("Quit").clicked() {
                            self.commands
                                .queue_command(Box::new(ExitCommand {}))
                                .unwrap();
                        }
                    });

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
    }

    fn left_panel(&mut self, resources: &mut Resources) {
        let ctx = &resources.gui.context.clone();
        egui::SidePanel::left("scene_explorer")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Tools");

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
        graph: &mut EntitySceneGraph,
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
            egui::CollapsingHeader::new(header)
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

        if response.clicked()
            && (!self.selected_entities.contains(&entity) || !self.selected_entities.is_empty())
        {
            self.selected_entities = vec![entity];
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
        egui::TopBottomPanel::bottom("assets")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Assets");
                ui.allocate_space(ui.available_size());
            });
    }

    fn viewport_panel(&mut self, resources: &mut Resources) {
        let context = &resources.gui.context;

        egui::Area::new("Viewport")
            .fixed_pos((0.0, 0.0))
            .show(context, |ui| {
                ui.with_layer_id(LayerId::background(), |ui| {
                    for entity in self.selected_entities.iter() {
                        let (projection, view) = resources
                            .world
                            .active_camera_matrices(resources.system.aspect_ratio())
                            .expect("Failed to get camera matrices!");
                        let transform = resources
                            .world
                            .entity_global_transform(*entity)
                            .expect("Failed to get entity transform!");
                        if let Some(gizmo_result) =
                            self.gizmo.render(ui, transform.matrix(), view, projection)
                        {
                            let model_matrix: glm::Mat4 = gizmo_result.transform.into();
                            let gizmo_transform = Transform::from(model_matrix);
                            let mut entry = resources.world.ecs.entry_mut(*entity).unwrap();
                            let mut transform = entry.get_component_mut::<Transform>().unwrap();
                            transform.translation = gizmo_transform.translation;
                            transform.rotation = gizmo_transform.rotation;
                            transform.scale = gizmo_transform.scale;
                            if entry.get_component::<RigidBody>().is_ok() {
                                resources
                                    .world
                                    .sync_rigid_body_to_transform(*entity)
                                    .expect("Failed to sync rigid body to transform!");
                            }
                        }
                    }
                });
            });
    }
}

impl State for Editor {
    fn label(&self) -> String {
        "Phantom Editor - Main".to_string()
    }

    fn on_start(&mut self, resources: &mut Resources) -> StateResult<()> {
        resources.config.graphics.debug_grid_active = true;
        Ok(())
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
        self.viewport_panel(resources);
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

    // fn on_mouse(
    //     &mut self,
    //     resources: &mut Resources,
    //     button: &MouseButton,
    //     button_state: &ElementState,
    // ) -> StateResult<Transition> {
    //     log::trace!("Mouse event: {:#?} {:#?}", button, button_state);
    //     if (MouseButton::Left, ElementState::Pressed) == (*button, *button_state) {
    //         let interact_distance = f32::MAX;
    //         let picked_entity = resources.world.pick_object(
    //             &resources.mouse_ray_configuration()?,
    //             interact_distance,
    //             EDITOR_COLLISION_GROUP,
    //         )?;
    //         if let Some(entity) = picked_entity {
    //             self.select_entity(entity, resources)?;
    //         }
    //     }
    //     Ok(Transition::None)
    // }

    fn on_key(
        &mut self,
        _resources: &mut Resources,
        input: KeyboardInput,
    ) -> StateResult<Transition> {
        log::trace!("Key event received: {:#?}", input);
        match (input.virtual_keycode, input.state) {
            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                self.selected_entities.clear();
            }
            (Some(VirtualKeyCode::T), ElementState::Pressed) => {
                self.gizmo.mode = GizmoMode::Translate;
            }
            (Some(VirtualKeyCode::R), ElementState::Pressed) => {
                self.gizmo.mode = GizmoMode::Rotate;
            }
            (Some(VirtualKeyCode::S), ElementState::Pressed) => {
                self.gizmo.mode = GizmoMode::Scale;
            }
            (Some(VirtualKeyCode::G), ElementState::Pressed) => {
                self.gizmo.orientation = match self.gizmo.orientation {
                    GizmoOrientation::Global => GizmoOrientation::Local,
                    GizmoOrientation::Local => GizmoOrientation::Global,
                }
            }
            _ => {}
        }
        Ok(Transition::None)
    }
}
