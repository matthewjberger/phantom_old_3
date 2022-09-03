use phantom_dependencies::{
    legion::{self, world::EntityAccessError},
    log,
    petgraph::{graph::WalkNeighbors, prelude::*},
    serde::{Deserialize, Serialize},
};
use std::{
    cmp::PartialEq,
    fmt::Debug,
    ops::{Index, IndexMut},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SceneGraphError {
    #[error("Failed to match the provided image format to a vulkan format!")]
    DetermineImageFormat,

    #[error("Failed to access entity!")]
    AccessEntity(#[from] EntityAccessError),

    #[error("Failed to walk scene graph!")]
    WalkSceneGraph(#[source] Box<dyn std::error::Error>),
}

type Result<T, E = SceneGraphError> = std::result::Result<T, E>;

pub type Ecs = legion::World;
pub type Entity = legion::Entity;

pub type EntitySceneGraph = SceneGraph<Entity>;
pub type EntitySceneGraphNode = SceneGraphNode<Entity>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct SceneGraph<T: Copy + PartialEq + Debug>(pub Graph<T, ()>);

impl<T> Default for SceneGraph<T>
where
    T: Copy + PartialEq + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SceneGraph<T>
where
    T: Copy + PartialEq + Debug,
{
    pub fn new() -> Self {
        Self(Graph::<T, ()>::new())
    }

    pub fn number_of_nodes(&self) -> usize {
        self.0.raw_nodes().len()
    }

    pub fn add_node(&mut self, node: T) -> NodeIndex {
        self.0.add_node(node)
    }

    pub fn remove_node(&mut self, node_index: NodeIndex) {
        log::info!("Removing node: {:#?}", node_index);
        let _ = self.0.remove_node(node_index);

        while let Some(child_index) = self.neighbors(node_index, Outgoing).next_node(&self.0) {
            self.remove_node(child_index);
        }
    }

    pub fn add_edge(&mut self, parent_node: NodeIndex, node: NodeIndex) {
        let _edge_index = self.0.add_edge(parent_node, node, ());
    }

    pub fn root_node_indices(&self) -> Result<Vec<NodeIndex>> {
        Ok(self
            .0
            .node_indices()
            .filter(|node_index| !self.has_parents(*node_index))
            .collect::<Vec<_>>())
    }

    pub fn root_nodes(&self) -> Result<Vec<SceneGraphNode<T>>> {
        Ok(self
            .root_node_indices()?
            .iter()
            .enumerate()
            .map(|(offset, node_index)| SceneGraphNode::new(self[*node_index], offset as _))
            .collect::<Vec<_>>())
    }

    pub fn collect_nodes(&self) -> Result<Vec<SceneGraphNode<T>>> {
        let mut nodes = Vec::new();
        let mut linear_offset = 0;
        self.walk(|node_index| {
            nodes.push(SceneGraphNode::new(self[node_index], linear_offset));
            linear_offset += 1;
            Ok(())
        })?;
        Ok(nodes)
    }

    pub fn get_parent_of(&self, index: NodeIndex) -> Option<NodeIndex> {
        let mut incoming_walker = self.0.neighbors_directed(index, Incoming).detach();
        incoming_walker.next_node(&self.0)
    }

    pub fn walk(
        &self,
        mut action: impl FnMut(NodeIndex) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<()> {
        for node_index in self.0.node_indices() {
            if self.has_parents(node_index) {
                continue;
            }
            let mut dfs = Dfs::new(&self.0, node_index);
            while let Some(node_index) = dfs.next(&self.0) {
                action(node_index).map_err(SceneGraphError::WalkSceneGraph)?;
            }
        }
        Ok(())
    }

    pub fn has_neighbors(&self, index: NodeIndex) -> bool {
        self.has_parents(index) || self.has_children(index)
    }

    pub fn has_parents(&self, index: NodeIndex) -> bool {
        self.neighbors(index, Incoming).next_node(&self.0).is_some()
    }

    pub fn has_children(&self, index: NodeIndex) -> bool {
        self.neighbors(index, Outgoing).next_node(&self.0).is_some()
    }

    pub fn neighbors(&self, index: NodeIndex, direction: Direction) -> WalkNeighbors<u32> {
        self.0.neighbors_directed(index, direction).detach()
    }

    pub fn find_node(&self, item: T) -> Option<NodeIndex> {
        self.0.node_indices().find(|i| self[*i] == item)
    }
}

impl<T> Index<NodeIndex> for SceneGraph<T>
where
    T: Copy + PartialEq + Debug,
{
    type Output = T;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<NodeIndex> for SceneGraph<T>
where
    T: Copy + PartialEq + Debug,
{
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub struct SceneGraphNode<T> {
    pub value: T,
    pub offset: u32,
}

impl<T> SceneGraphNode<T> {
    pub fn new(value: T, offset: u32) -> Self {
        Self { value, offset }
    }
}
