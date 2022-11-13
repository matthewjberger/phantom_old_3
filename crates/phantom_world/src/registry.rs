use crate::{Camera, Ecs, Light, MeshRender, Name, RigidBody, Skin, Transform, World};
use phantom_dependencies::{
    bincode,
    lazy_static::lazy_static,
    legion::{
        self,
        serialize::{set_entity_serializer, Canon},
        storage::Component,
        Registry,
    },
    serde::{de::DeserializeSeed, Deserialize, Deserializer, Serialize, Serializer},
    thiserror::Error,
};
use std::sync::{Arc, RwLock};

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Failed to deserialized world!")]
    DeserializeWorld(#[source] bincode::Error),

    #[error("Failed to deserialized world!")]
    SerializeWorld(#[source] bincode::Error),

    #[error("Failed to access component registry!")]
    AccessComponentRegistry,
}

type Result<T, E = RegistryError> = std::result::Result<T, E>;

lazy_static! {
    pub static ref COMPONENT_REGISTRY: Arc<RwLock<Registry<String>>> = {
        let mut registry = Registry::default();
        registry.register::<Name>("name".to_string());
        registry.register::<Transform>("transform".to_string());
        registry.register::<Camera>("camera".to_string());
        registry.register::<MeshRender>("mesh".to_string());
        registry.register::<Skin>("skin".to_string());
        registry.register::<Light>("light".to_string());
        registry.register::<RigidBody>("rigid_body".to_string());
        Arc::new(RwLock::new(registry))
    };
    pub static ref ENTITY_SERIALIZER: Canon = Canon::default();
}

pub fn register_component<T: Component + Serialize + for<'de> Deserialize<'de>>(
    key: &str,
) -> Result<()> {
    let mut registry = COMPONENT_REGISTRY
        .write()
        .map_err(|_| RegistryError::AccessComponentRegistry)?;
    registry.register::<T>(key.to_string());
    Ok(())
}

pub fn serialize_ecs<S>(ecs: &Ecs, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let registry = (*COMPONENT_REGISTRY)
        .read()
        .expect("Failed to get the component registry lock!");
    ecs.as_serializable(legion::any(), &*registry, &*ENTITY_SERIALIZER)
        .serialize(serializer)
}

pub fn deserialize_ecs<'de, D>(deserializer: D) -> Result<Ecs, D::Error>
where
    D: Deserializer<'de>,
{
    (*COMPONENT_REGISTRY)
        .read()
        .expect("Failed to get the component registry lock!")
        .as_deserialize(&*ENTITY_SERIALIZER)
        .deserialize(deserializer)
}

pub fn world_as_bytes(world: &World) -> Result<Vec<u8>> {
    set_entity_serializer(&*ENTITY_SERIALIZER, || bincode::serialize(world))
        .map_err(RegistryError::SerializeWorld)
}

pub fn world_from_bytes(bytes: &[u8]) -> Result<World> {
    set_entity_serializer(&*ENTITY_SERIALIZER, || bincode::deserialize(bytes))
        .map_err(RegistryError::DeserializeWorld)
}
