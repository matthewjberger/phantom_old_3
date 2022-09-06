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

    pub fn add_root_node(&mut self, node: T) -> NodeIndex {
        self.0.add_node(node)
    }

    pub fn add_child(&mut self, parent_index: NodeIndex, value: T) -> NodeIndex {
        log::info!("Adding child node to parent: {:#?}", parent_index);
        let child_index = self.add_root_node(value);
        self.add_edge(parent_index, child_index);
        child_index
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

#[derive(Debug, PartialEq, Eq)]
pub struct SceneGraphNode<T> {
    pub value: T,
    pub offset: u32,
}

impl<T> SceneGraphNode<T> {
    pub fn new(value: T, offset: u32) -> Self {
        Self { value, offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_root_nodes() -> Result<()> {
        let (scenegraph, _first_node_index, _second_node_index) = create_scenegraph();

        let root_nodes = scenegraph.root_nodes()?;
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(
            root_nodes.iter().next(),
            Some(&SceneGraphNode::<i32> {
                value: FIRST_VALUE,
                offset: 0
            })
        );

        Ok(())
    }

    #[test]
    fn get_root_node_indices() -> Result<()> {
        let (scenegraph, first_node_index, _second_node_index) = create_scenegraph();

        let root_nodes = scenegraph.root_node_indices()?;
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes.iter().next(), Some(&first_node_index));

        Ok(())
    }

    #[test]
    fn get_parent_of() -> Result<()> {
        let (scenegraph, first_node_index, second_node_index) = create_scenegraph();
        assert_eq!(
            scenegraph.get_parent_of(second_node_index),
            Some(first_node_index),
        );
        Ok(())
    }

    #[test]
    fn add_child() -> Result<()> {
        let (mut scenegraph, first_node_index, second_node_index) = create_scenegraph();

        let child_node_index = scenegraph.add_child(first_node_index, 18);
        assert_eq!(scenegraph.number_of_nodes(), 3);
        assert_eq!(
            scenegraph.get_parent_of(child_node_index),
            Some(first_node_index),
        );

        let second_child_node_index = scenegraph.add_child(second_node_index, 34);
        assert_eq!(scenegraph.number_of_nodes(), 4);
        assert_eq!(
            scenegraph.get_parent_of(second_child_node_index),
            Some(second_node_index),
        );

        Ok(())
    }

    #[test]
    fn remove_node() -> Result<()> {
        let (mut scenegraph, _first_node_index, second_node_index) = create_scenegraph();

        scenegraph.remove_node(second_node_index);
        assert_eq!(scenegraph.number_of_nodes(), 1);

        Ok(())
    }

    #[test]
    fn collect_nodes() -> Result<()> {
        let (scenegraph, _first_node_index, _second_node_index) = create_scenegraph();

        let nodes = scenegraph.collect_nodes()?;
        assert_eq!(nodes.len(), 2);
        assert_eq!(
            nodes.iter().next(),
            Some(&SceneGraphNode::<i32> {
                value: FIRST_VALUE,
                offset: 0
            })
        );
        assert_eq!(
            nodes.iter().skip(1).next(),
            Some(&SceneGraphNode::<i32> {
                value: SECOND_VALUE,
                offset: 1
            })
        );

        Ok(())
    }

    #[test]
    fn walk() -> Result<()> {
        let (scenegraph, _first_node_index, _second_node_index) = create_scenegraph();

        scenegraph.walk(|node_index| {
            let expected_value = match node_index.index() {
                0 => FIRST_VALUE,
                1 => SECOND_VALUE,
                value => panic!("An invalid node index was reached! {}", value),
            };
            assert_eq!(scenegraph[node_index], expected_value);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn find_node() -> Result<()> {
        let (scenegraph, first_node_index, second_node_index) = create_scenegraph();
        assert_eq!(scenegraph.find_node(FIRST_VALUE), Some(first_node_index));
        assert_eq!(scenegraph.find_node(SECOND_VALUE), Some(second_node_index));
        Ok(())
    }

    #[test]
    fn has_neighbors() -> Result<()> {
        let (scenegraph, first_node_index, second_node_index) = create_scenegraph();
        assert_eq!(scenegraph.has_neighbors(first_node_index), true);
        assert_eq!(scenegraph.has_neighbors(second_node_index), true);
        Ok(())
    }

    #[test]
    fn has_parents() -> Result<()> {
        let (scenegraph, first_node_index, second_node_index) = create_scenegraph();
        assert_eq!(scenegraph.has_parents(first_node_index), false);
        assert_eq!(scenegraph.has_parents(second_node_index), true);
        Ok(())
    }

    #[test]
    fn has_children() -> Result<()> {
        let (scenegraph, first_node_index, second_node_index) = create_scenegraph();
        assert_eq!(scenegraph.has_children(first_node_index), true);
        assert_eq!(scenegraph.has_children(second_node_index), false);
        Ok(())
    }

    const FIRST_VALUE: i32 = 4;
    const SECOND_VALUE: i32 = 12;

    fn create_scenegraph() -> (SceneGraph<i32>, NodeIndex, NodeIndex) {
        // 0
        //  \
        //   1
        let mut scenegraph = SceneGraph::new();
        let first_node_index = scenegraph.add_root_node(4);
        let second_node_index = scenegraph.add_root_node(12);
        scenegraph.add_edge(first_node_index, second_node_index);
        (scenegraph, first_node_index, second_node_index)
    }
}
