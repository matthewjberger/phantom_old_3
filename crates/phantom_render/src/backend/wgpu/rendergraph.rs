use phantom_dependencies::{
    log,
    petgraph::{dot::Dot, Graph},
    wgpu,
};
use std::{collections::HashMap, fmt::Debug};

pub fn create_rendergraph() -> RenderGraph {
    let mut rendergraph = RenderGraph::new();

    // Add default texture
    rendergraph.add_resource(label)

    rendergraph
}

pub trait Node {
    fn label(&self) -> String {
        "Unnamed Node".to_string()
    }
    fn run(&self) {}
    fn inputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
}

impl Debug for dyn Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub type Handle = String;
pub type ResourceMap = HashMap<Handle, Resource>;

#[derive(Default)]
pub struct RenderGraph {
    graph: Graph<Box<dyn Node>, Vec<Handle>>,
    resources: ResourceMap,
    next_available_index: usize,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print_graph(&self) {
        log::info!("Rendergraph:\n{:#?}", Dot::with_config(&self.graph, &[]));
    }

    pub fn add_node(&mut self, node: impl Node + Copy + 'static) {
        let node_index = self.graph.add_node(Box::new(node));
    }

    pub fn import_resource(&mut self, label: &str, resource: Resource) {

    }

    pub fn execute(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        // Iterate over nodes and create edges made of resources to connect them
        // Topologically sort nodes
        // Execute nodes in order
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Resource {
    name: String,
    kind: ResourceKind,
}

impl Resource {
    pub fn get(&self) -> &ResourceKind {
        &self.kind
    }
}

#[derive(Debug)]
pub enum ResourceKind {
    Buffer(wgpu::Buffer),
    TextureView(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}
