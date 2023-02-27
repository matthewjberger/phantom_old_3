use crate::{
    deserialize_ecs, scenegraph, serialize_ecs, world_as_bytes, world_from_bytes, Animation,
    Camera, Ecs, Entity, EntitySceneGraph, EntitySceneGraphNode, Material, Name, PerspectiveCamera,
    Projection, RegistryError, RigidBody, SceneGraphError, Texture, TextureError, Transform,
    WorldPhysics,
};
use bmfont::{self, BMFont, OrdinateOrientation};
use legion::{
    world::{ComponentError, EntityAccessError},
    EntityStore, IntoQuery,
};
use nalgebra::{Point, Point3};
use nalgebra_glm as glm;
use petgraph::prelude::*;
use rapier3d::{
    dynamics::RigidBodyBuilder,
    geometry::{ColliderBuilder, InteractionGroups, Ray},
    prelude::{Collider, QueryFilter, RigidBodyType},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    marker::{Send, Sync},
    ops::Range,
    path::Path,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Failed to find entity!")]
    FindEntity,

    #[error("Failed to acquire physics body to render!")]
    GetPhysicsBody,

    #[error("A transform component was requested from a component that does not have one!")]
    RequestTransform(#[source] ComponentError),

    #[error("Failed to access entity!")]
    AccessEntity(#[from] EntityAccessError),

    #[error("Failed to get transform!")]
    GetComponent(#[from] ComponentError),

    #[error("There must be at least one entity with an enabled camera component to render with!")]
    FindActiveCamera,

    #[error("Failed to walk the scene graph!")]
    WalkEntitySceneGraph(#[source] scenegraph::SceneGraphError),

    #[error("Failed to lookup material at index: `{0}`")]
    LookupMaterial(usize),

    #[error("Failed to decode SDF font from file!")]
    DecodeBitmapFontFromFile(#[source] bmfont::Error),

    #[error("Failed to load SDF font  file!")]
    LoadSdfFontFile(#[source] std::io::Error),

    #[error("Failed to load SDF texture from file!")]
    LoadSdfTextureFromFile(#[source] TextureError),

    #[error("Failed to load HDR image from file!")]
    LoadHdr(#[source] TextureError),

    #[error("Failed to load world from file!")]
    LoadWorldFromFile(#[source] std::io::Error),

    #[error("Failed to get a collider's parent!")]
    GetColliderParent,

    #[error("Failed to save world!")]
    SaveWorldToFile(#[source] std::io::Error),

    #[error("Failed to serialize world!")]
    SerializeWorld(#[source] RegistryError),

    #[error("Failed to deserialize world!")]
    DeserializeWorld(#[source] RegistryError),

    #[error("Failed to find default scenegraph in scene named '`{0}`'!")]
    FindDefaultScenegraph(String),

    #[error("Failed to global transform of node in scene graph!")]
    GetGlobalTransform(#[source] SceneGraphError),
}

type Result<T, E = WorldError> = std::result::Result<T, E>;

#[derive(Default, Serialize, Deserialize)]
pub struct World {
    #[serde(serialize_with = "serialize_ecs", deserialize_with = "deserialize_ecs")]
    pub ecs: Ecs,
    pub physics: WorldPhysics,
    pub scene: Scene,
    pub animations: Vec<Animation>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub hdr_textures: Vec<Texture>,
    pub geometry: Geometry,
    pub fonts: HashMap<String, SdfFont>,
}

impl World {
    pub const MAIN_CAMERA_NAME: &'static str = "Main Camera";

    pub fn new() -> Result<World> {
        let mut world = World::default();
        world.initialize()?;
        Ok(world)
    }

    fn initialize(&mut self) -> Result<()> {
        self.scene = Scene::default();
        self.scene.name = "Main Scene".to_string();
        self.add_default_camera()?;
        self.add_default_light()?;
        Ok(())
    }

    pub fn add_default_camera(&mut self) -> Result<()> {
        let position = glm::vec3(0.0, 0.0, 10.0);
        let mut transform = Transform {
            translation: position,
            ..Default::default()
        };
        transform.look_at(&(-position), &glm::Vec3::y());

        let camera_entity = self.ecs.push((
            Name("Default Camera".to_string()),
            transform,
            Camera {
                name: Self::MAIN_CAMERA_NAME.to_string(),
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 70_f32.to_radians(),
                    z_far: Some(1000.0),
                    z_near: 0.1,
                }),
                enabled: true,
            },
        ));

        self.scene
            .default_scenegraph_mut()?
            .add_root_node(camera_entity);

        Ok(())
    }

    pub fn add_default_light(&mut self) -> Result<()> {
        let position = glm::vec3(-4.0, 2.0, 0.0);
        let mut transform = Transform {
            translation: position,
            ..Default::default()
        };
        transform.look_at(&(-position), &glm::Vec3::y());
        let light_entity = self.ecs.push((
            Name("Default Light".to_string()),
            transform,
            Light {
                color: glm::vec3(1.0, 1.0, 1.0),
            },
        ));
        self.scene
            .default_scenegraph_mut()?
            .add_root_node(light_entity);
        Ok(())
    }

    pub fn active_camera(&self) -> Result<Entity> {
        let mut query = <(Entity, &Camera)>::query();
        for (entity, camera) in query.iter(&self.ecs) {
            if camera.enabled {
                return Ok(*entity);
            }
        }
        Err(WorldError::FindActiveCamera)
    }

    pub fn global_transform(
        &self,
        graph: &EntitySceneGraph,
        index: NodeIndex,
    ) -> Result<glm::Mat4> {
        let entity = graph[index];
        let transform = match self.ecs.entry_ref(entity)?.get_component::<Transform>() {
            Ok(transform) => transform.matrix(),
            Err(error) => return Err(WorldError::RequestTransform(error)),
        };
        let mut incoming_walker = graph.0.neighbors_directed(index, Incoming).detach();
        match incoming_walker.next_node(&graph.0) {
            Some(parent_index) => Ok(self.global_transform(graph, parent_index)? * transform),
            None => Ok(transform),
        }
    }

    pub fn entity_global_transform_matrix(&self, entity: Entity) -> Result<glm::Mat4> {
        let mut transform = glm::Mat4::identity();
        let mut found = false;
        for graph in self.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    if entity != graph[node_index] {
                        return Ok(());
                    }
                    transform = self.global_transform(graph, node_index)?;
                    found = true;
                    Ok(())
                })
                .map_err(WorldError::WalkEntitySceneGraph)?;
            if found {
                break;
            }
        }
        if !found {
            // TODO: Maybe returning an error if the global transform of an entity that isn't in the scenegraph is better...
            // Not found in the scenegraph, so the entity just has a local transform
            transform = self
                .ecs
                .entry_ref(entity)?
                .get_component::<Transform>()?
                .matrix();
        }
        Ok(transform)
    }

    pub fn entity_global_transform(&self, entity: Entity) -> Result<Transform> {
        let transform_matrix = self.entity_global_transform_matrix(entity)?;
        Ok(Transform::from(transform_matrix))
    }

    pub fn active_camera_matrices(&self, aspect_ratio: f32) -> Result<(glm::Mat4, glm::Mat4)> {
        let camera_entity = self.active_camera()?;
        let transform = self.entity_global_transform(camera_entity)?;
        let view = transform.as_view_matrix();
        let projection = {
            let entry = self.ecs.entry_ref(camera_entity)?;
            let camera = entry.get_component::<Camera>()?;
            camera.projection_matrix(aspect_ratio)
        };
        Ok((projection, view))
    }

    pub fn active_camera_is_main(&self) -> Result<bool> {
        let entity = self.active_camera()?;
        let entry = self.ecs.entry_ref(entity)?;
        let camera = entry.get_component::<Camera>()?;
        Ok(camera.name == Self::MAIN_CAMERA_NAME)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.ecs.clear();
        self.scene.graphs.clear();
        self.textures.clear();
        self.animations.clear();
        self.materials.clear();
        self.geometry.clear();
        self.initialize()?;
        Ok(())
    }

    pub fn material_at_index(&self, index: usize) -> Result<&Material> {
        self.materials
            .get(index)
            .ok_or(WorldError::LookupMaterial(index))
    }

    pub fn components<T: Send + Sync + Copy + Clone + 'static>(
        &self,
    ) -> Result<Vec<(Transform, T)>> {
        let mut components = Vec::new();
        for graph in self.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    let entity = graph[node_index];
                    let node_transform = self.global_transform(graph, node_index)?;
                    if let Ok(component) = self.ecs.entry_ref(entity)?.get_component::<T>() {
                        components.push((Transform::from(node_transform), *component));
                    }
                    Ok(())
                })
                .map_err(WorldError::WalkEntitySceneGraph)?;
        }
        Ok(components)
    }

    pub fn joint_matrices(&self) -> Result<Vec<glm::Mat4>> {
        let mut offset = 0;
        let mut number_of_joints = 0;
        for graph in self.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    let entity = graph[node_index];
                    if let Ok(skin) = self.ecs.entry_ref(entity)?.get_component::<Skin>() {
                        number_of_joints += skin.joints.len();
                    }
                    Ok(())
                })
                .map_err(WorldError::WalkEntitySceneGraph)?;
        }
        let mut joint_matrices = vec![glm::Mat4::identity(); number_of_joints];
        for graph in self.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    let entity = graph[node_index];
                    let node_transform = self.global_transform(graph, node_index)?;
                    if let Ok(skin) = self.ecs.entry_ref(entity)?.get_component::<Skin>() {
                        for joint in skin.joints.iter() {
                            let joint_transform = {
                                let mut transform = glm::Mat4::identity();
                                for graph in self.scene.graphs.iter() {
                                    if let Some(index) = graph.find_node(joint.target) {
                                        transform = self.global_transform(graph, index)?;
                                    }
                                }
                                transform
                            };
                            joint_matrices[offset] = glm::inverse(&node_transform)
                                * joint_transform
                                * joint.inverse_bind_matrix;
                            offset += 1;
                        }
                    }
                    Ok(())
                })
                .map_err(WorldError::WalkEntitySceneGraph)?;
        }
        Ok(joint_matrices)
    }

    pub fn add_cylinder_collider(
        &mut self,
        entity: Entity,
        half_height: f32,
        radius: f32,
        collision_groups: InteractionGroups,
    ) -> Result<()> {
        let collider = ColliderBuilder::cylinder(half_height, radius)
            .collision_groups(collision_groups)
            .build();
        self.insert_collider(entity, collider)?;
        Ok(())
    }

    pub fn add_box_collider(
        &mut self,
        entity: Entity,
        collision_groups: InteractionGroups,
    ) -> Result<()> {
        let bounding_box = {
            let entry = self.ecs.entry_ref(entity)?;
            let mesh = entry.get_component::<MeshRender>()?;
            self.geometry.meshes[&mesh.name].bounding_box()
        };
        let entry = self.ecs.entry_ref(entity)?;
        let transform = entry.get_component::<Transform>()?;
        let half_extents = bounding_box.half_extents().component_mul(&transform.scale);
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .collision_groups(collision_groups)
            .build();
        self.insert_collider(entity, collider)?;
        Ok(())
    }

    pub fn add_capsule_collider(
        &mut self,
        entity: Entity,
        collision_groups: InteractionGroups,
    ) -> Result<()> {
        let bounding_box = {
            let entry = self.ecs.entry_ref(entity)?;
            let mesh = entry.get_component::<MeshRender>()?;
            self.geometry.meshes[&mesh.name].bounding_box()
        };

        let entry = self.ecs.entry_ref(entity)?;
        let transform = entry.get_component::<Transform>()?;
        let half_extents = bounding_box.half_extents().component_mul(&transform.scale);
        let collider = ColliderBuilder::capsule_y(
            half_extents.y,
            std::cmp::max(half_extents.x as u32, half_extents.z as u32) as f32,
        )
        .collision_groups(collision_groups)
        .build();

        self.insert_collider(entity, collider)?;

        Ok(())
    }

    fn insert_collider(&mut self, entity: Entity, collider: Collider) -> Result<(), WorldError> {
        match self.ecs.entry_mut(entity)?.get_component_mut::<RigidBody>() {
            Ok(rigid_body) => {
                let handle = self.physics.colliders.insert_with_parent(
                    collider,
                    rigid_body.handle,
                    &mut self.physics.bodies,
                );
                rigid_body.colliders.push(handle);
            }
            Err(_) => {
                self.physics.colliders.insert(collider);
            }
        };
        Ok(())
    }

    pub fn add_trimesh_collider(
        &mut self,
        entity: Entity,
        collision_groups: InteractionGroups,
    ) -> Result<()> {
        let entry = self.ecs.entry_ref(entity)?;
        let mesh = entry.get_component::<MeshRender>()?;
        let transform = self.entity_global_transform(entity)?;
        let mesh = &self.geometry.meshes[&mesh.name];

        let mut colliders = Vec::new();

        for primitive in mesh.primitives.iter() {
            let vertices = self.geometry.vertices
                [primitive.first_vertex..primitive.first_vertex + primitive.number_of_vertices]
                .iter()
                .map(|v| Point::from_slice((v.position.component_mul(&transform.scale)).as_slice()))
                .collect::<Vec<_>>();

            let indices = self.geometry.indices
                [primitive.first_index..primitive.first_index + primitive.number_of_indices]
                .chunks(3)
                .map(|chunk| {
                    [
                        chunk[0] - primitive.first_vertex as u32,
                        chunk[1] - primitive.first_vertex as u32,
                        chunk[2] - primitive.first_vertex as u32,
                    ]
                })
                .collect::<Vec<[u32; 3]>>();

            let collider = ColliderBuilder::trimesh(vertices, indices)
                .collision_groups(collision_groups)
                .build();

            colliders.push(collider);
        }

        colliders
            .into_iter()
            .try_for_each(|collider| self.insert_collider(entity, collider))?;
        Ok(())
    }

    pub fn add_rigid_body(&mut self, entity: Entity, rigid_body_type: RigidBodyType) -> Result<()> {
        let handle = {
            let isometry =
                Transform::from(self.entity_global_transform_matrix(entity)?).as_isometry();

            // Insert a corresponding rigid body
            let rigid_body = RigidBodyBuilder::new(rigid_body_type)
                .position(isometry)
                .build();
            self.physics.bodies.insert(rigid_body)
        };
        self.ecs
            .entry(entity)
            .ok_or(WorldError::FindEntity)?
            .add_component(RigidBody::new(handle));
        Ok(())
    }

    pub fn remove_rigid_body(&mut self, entity: Entity) -> Result<()> {
        let mut entry = self.ecs.entry(entity).ok_or(WorldError::FindEntity)?;
        let rigid_body_handle = entry.get_component::<RigidBody>()?.handle;
        entry.remove_component::<RigidBody>();
        self.physics.remove_rigid_body(rigid_body_handle);
        Ok(())
    }

    pub fn flatten_scenegraphs(&self) -> Vec<EntitySceneGraphNode> {
        let mut offset = 0;
        self.scene
            .graphs
            .iter()
            .flat_map(|graph| {
                let mut graph_nodes = graph.collect_nodes().expect("Failed to collect nodes");
                graph_nodes
                    .iter_mut()
                    .for_each(|node| node.offset += offset);
                offset += graph_nodes.len() as u32;
                graph_nodes
            })
            .collect::<Vec<_>>()
    }

    pub fn mouse_ray(&mut self, configuration: &MouseRayConfiguration) -> Result<Ray> {
        let MouseRayConfiguration {
            viewport,
            projection_matrix,
            view_matrix,
            mouse_position,
        } = *configuration;

        let mut position = mouse_position;
        position.y = viewport.height - position.y;

        let near_point = glm::vec2_to_vec3(&position);

        let mut far_point = near_point;
        far_point.z = 1.0;

        let viewport = viewport.as_glm_vec();
        let p_near = glm::unproject_zo(&near_point, &view_matrix, &projection_matrix, viewport);
        let p_far = glm::unproject_zo(&far_point, &view_matrix, &projection_matrix, viewport);

        let direction = (p_far - p_near).normalize();
        let ray = Ray::new(Point3::from(p_near), direction);

        Ok(ray)
    }

    pub fn pick_object(
        &mut self,
        mouse_ray_configuration: &MouseRayConfiguration,
        interact_distance: f32,
        groups: InteractionGroups,
    ) -> Result<Option<Entity>> {
        let ray = self.mouse_ray(mouse_ray_configuration)?;

        let hit = self.physics.query_pipeline.cast_ray(
            &self.physics.bodies,
            &self.physics.colliders,
            &ray,
            interact_distance,
            true,
            QueryFilter::from(groups),
        );

        let mut picked_entity = None;
        if let Some((handle, _)) = hit {
            let collider = &self.physics.colliders[handle];
            let rigid_body_handle = collider.parent().ok_or(WorldError::GetColliderParent)?;
            let mut query = <(Entity, &RigidBody)>::query();
            for (entity, rigid_body) in query.iter(&self.ecs) {
                if rigid_body.handle == rigid_body_handle {
                    picked_entity = Some(*entity);
                    break;
                }
            }
        }

        Ok(picked_entity)
    }

    pub fn tick(&mut self, delta_time: f32) -> Result<()> {
        self.physics.update(delta_time);
        self.sync_all_rigid_bodies();
        Ok(())
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        world_as_bytes(self).map_err(WorldError::SerializeWorld)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<World> {
        world_from_bytes(bytes).map_err(WorldError::DeserializeWorld)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.as_bytes()?).map_err(WorldError::SaveWorldToFile)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_bytes(&std::fs::read(path).map_err(WorldError::LoadWorldFromFile)?)
    }

    pub fn load_hdr(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.hdr_textures
            .push(Texture::from_hdr(path).map_err(WorldError::LoadHdr)?);
        Ok(())
    }

    /// Sync the entity's physics rigid body with its transform
    pub fn sync_rigid_body_to_transform(&mut self, entity: Entity) -> Result<()> {
        let entry = self.ecs.entry_ref(entity)?;
        let rigid_body = entry.get_component::<RigidBody>()?;
        let transform = entry.get_component::<Transform>()?;
        if let Some(body) = self.physics.bodies.get_mut(rigid_body.handle) {
            let mut position = *body.position();
            position.translation.vector = transform.translation;
            body.set_position(position, true);
        }
        Ok(())
    }

    /// Sync the entity's transform with its physics rigid body
    pub fn sync_transform_to_rigid_body(&mut self, entity: Entity) -> Result<()> {
        let rigid_body_handle = self
            .ecs
            .entry_ref(entity)?
            .get_component::<RigidBody>()?
            .handle;
        let mut entry = self.ecs.entry(entity).ok_or(WorldError::FindEntity)?;
        let transform = entry.get_component_mut::<Transform>()?;
        if let Some(body) = self.physics.bodies.get(rigid_body_handle) {
            let position = body.position();
            transform.translation = position.translation.vector;
            transform.rotation = *position.rotation.quaternion();
        }
        if let Some(body) = self.physics.bodies.get_mut(rigid_body_handle) {
            body.wake_up(true);
        }
        Ok(())
    }

    /// Sync the render transforms with the physics rigid bodies
    pub fn sync_all_rigid_bodies(&mut self) {
        let mut query = <(&RigidBody, &mut Transform)>::query();
        for (rigid_body, transform) in query.iter_mut(&mut self.ecs) {
            if let Some(body) = self.physics.bodies.get(rigid_body.handle) {
                let position = body.position();
                transform.translation = position.translation.vector;
                transform.rotation = *position.rotation.quaternion();
            }
        }
    }

    pub fn entity_model_matrix(
        &self,
        entity: Entity,
        global_transform: glm::Mat4,
    ) -> Result<glm::Mat4> {
        let entry = self.ecs.entry_ref(entity)?;
        let model = match entry.get_component::<RigidBody>() {
            Ok(rigid_body) => {
                let body = self
                    .physics
                    .bodies
                    .get(rigid_body.handle)
                    .ok_or(WorldError::GetPhysicsBody)?;
                let position = body.position();
                let translation = position.translation.vector;
                let rotation = *position.rotation.quaternion();
                let scale = Transform::from(global_transform).scale;
                Transform::new(translation, rotation, scale).matrix()
            }
            Err(_) => global_transform,
        };
        Ok(model)
    }

    pub fn get_metadata(&self) -> Vec<EntityMetadata> {
        let mut metadata = Vec::new();
        let mut offset = -1;
        for graph in self.scene.graphs.iter() {
            graph
                .walk(|node_index| {
                    offset += 1;

                    let entity = graph[node_index];
                    let entry = self.ecs.entry_ref(entity)?;

                    let mesh_result = entry
                        .get_component::<MeshRender>()
                        .map(|mesh_render| self.geometry.meshes.get(&mesh_render.name));
                    if let Ok(Some(mesh)) = mesh_result {
                        for primitive in mesh.primitives.iter() {
                            let start = primitive.first_index as u32;
                            metadata.push(EntityMetadata {
                                index_range: start
                                    ..(primitive.first_index + primitive.number_of_indices) as u32,
                                offset: offset as _,
                            });
                        }
                    }

                    Ok(())
                })
                .unwrap();
        }
        metadata
    }
}

#[derive(Default, Copy, Clone)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        let height = if self.height > 0.0 { self.height } else { 1.0 };
        self.width / height
    }

    pub fn as_glm_vec(&self) -> glm::Vec4 {
        glm::vec4(self.x, self.y, self.width, self.height)
    }
}

pub struct MouseRayConfiguration {
    pub viewport: Viewport,
    pub projection_matrix: glm::Mat4,
    pub view_matrix: glm::Mat4,
    pub mouse_position: glm::Vec2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub graphs: Vec<EntitySceneGraph>,
    pub skybox: Option<usize>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: "Unnamed Scene".to_string(),
            graphs: vec![EntitySceneGraph::default()],
            skybox: None,
        }
    }
}

impl Scene {
    pub fn default_scenegraph_mut(&mut self) -> Result<&mut EntitySceneGraph> {
        match self.graphs.iter_mut().next() {
            Some(graph) => Ok(graph),
            None => Err(WorldError::FindDefaultScenegraph(self.name.to_string())),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Light {
    pub color: glm::Vec3,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LightKind {
    Directional,
    Point,
    Spot {
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    },
}

impl Default for LightKind {
    fn default() -> Self {
        Self::Directional
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skin {
    pub name: String,
    pub joints: Vec<Joint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Joint {
    pub target: Entity,
    pub inverse_bind_matrix: glm::Mat4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRender {
    pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
    pub weights: Vec<f32>,
}

impl Mesh {
    pub fn bounding_box(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::new_invalid();
        self.primitives
            .iter()
            .map(|primitive| &primitive.bounding_box)
            .for_each(|primitive_bounding_box| bounding_box.fit_box(primitive_bounding_box));
        bounding_box
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: glm::Vec3,
    pub max: glm::Vec3,
}

impl BoundingBox {
    pub fn new_invalid() -> Self {
        Self {
            min: glm::vec3(f32::MAX, f32::MAX, f32::MAX),
            max: glm::vec3(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn new(min: glm::Vec3, max: glm::Vec3) -> Self {
        Self { min, max }
    }

    pub fn extents(&self) -> glm::Vec3 {
        glm::abs(&(self.max - self.min))
    }

    pub fn half_extents(&self) -> glm::Vec3 {
        self.extents() / 2.0
    }

    pub fn center(&self) -> glm::Vec3 {
        self.min + self.half_extents()
    }

    pub fn fit_box(&mut self, bounding_box: &Self) {
        self.fit_point(bounding_box.min);
        self.fit_point(bounding_box.max);
    }

    pub fn fit_point(&mut self, point: glm::Vec3) {
        self.min.x = f32::min(self.min.x, point.x);
        self.min.y = f32::min(self.min.y, point.y);
        self.min.z = f32::min(self.min.z, point.z);

        self.max.x = f32::max(self.max.x, point.x);
        self.max.y = f32::max(self.max.y, point.y);
        self.max.z = f32::max(self.max.z, point.z);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    pub first_vertex: usize,
    pub first_index: usize,
    pub number_of_vertices: usize,
    pub number_of_indices: usize,
    pub material_index: Option<usize>,
    pub morph_targets: Vec<MorphTarget>,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTarget {
    pub positions: Vec<glm::Vec4>,
    pub normals: Vec<glm::Vec4>,
    pub tangents: Vec<glm::Vec4>,
}

impl MorphTarget {
    pub fn total_length(&self) -> usize {
        self.positions.len() + self.normals.len() + self.tangents.len()
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Geometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub meshes: HashMap<String, Mesh>,
}

impl Geometry {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub uv_0: glm::Vec2,
    pub uv_1: glm::Vec2,
    pub joint_0: glm::Vec4,
    pub weight_0: glm::Vec4,
    pub color_0: glm::Vec3,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: glm::Vec3::default(),
            normal: glm::Vec3::default(),
            uv_0: glm::Vec2::default(),
            uv_1: glm::Vec2::default(),
            joint_0: glm::Vec4::default(),
            weight_0: glm::Vec4::default(),
            color_0: glm::vec3(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SdfFont {
    texture: Texture,
    font: BMFont,
}

impl SdfFont {
    pub fn new(font_path: impl AsRef<Path>, texture_path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(font_path).map_err(WorldError::LoadSdfFontFile)?;
        let font = BMFont::new(file, OrdinateOrientation::TopToBottom)
            .map_err(WorldError::DecodeBitmapFontFromFile)?;
        let texture =
            Texture::from_file(texture_path).map_err(WorldError::LoadSdfTextureFromFile)?;
        Ok(Self { texture, font })
    }
}

#[derive(Default)]
pub struct EntityMetadata {
    pub index_range: Range<u32>,
    pub offset: u32,
}
