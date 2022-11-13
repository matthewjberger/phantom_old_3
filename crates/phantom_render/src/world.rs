use phantom_dependencies::{anyhow::Result, legion::EntityStore};
use phantom_world::{MeshRender, World};
use std::ops::Range;

#[derive(Default)]
pub(crate) struct RenderJob {
    pub index_range: Range<u32>,
    pub entity_offset: u32,
}

pub(crate) fn create_jobs(world: &World) -> Result<Vec<RenderJob>> {
    let mut jobs = Vec::new();
    let mut offset = -1;
    for graph in world.scene.graphs.iter() {
        graph
            .walk(|node_index| {
                offset += 1;

                let entity = graph[node_index];
                let entry = world.ecs.entry_ref(entity)?;

                let mesh_result = entry
                    .get_component::<MeshRender>()
                    .map(|mesh_render| world.geometry.meshes.get(&mesh_render.name));
                if let Ok(Some(mesh)) = mesh_result {
                    for primitive in mesh.primitives.iter() {
                        let start = primitive.first_index as u32;
                        let job = RenderJob {
                            index_range: start
                                ..(primitive.first_index + primitive.number_of_indices) as u32,
                            entity_offset: offset as _,
                        };
                        jobs.push(job);
                    }
                }

                Ok(())
            })
            .unwrap();
    }

    Ok(jobs)
}
