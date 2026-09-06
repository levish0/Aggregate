use aggregate_geography::Heightfield;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub fn chunks(terrain: &Heightfield) -> Vec<Mesh> {
    let mut meshes = Vec::new();
    let chunk_cells = 32;
    for start_z in (0..terrain.rows).step_by(chunk_cells) {
        for start_x in (0..terrain.columns).step_by(chunk_cells) {
            let columns = (terrain.columns - start_x).min(chunk_cells as u32);
            let rows = (terrain.rows - start_z).min(chunk_cells as u32);
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut indices = Vec::new();
            for z in 0..=rows {
                for x in 0..=columns {
                    let gx = start_x + x;
                    let gz = start_z + z;
                    positions.push(terrain.position(gx, gz).to_array());
                    normals.push(terrain.normal(gx, gz).to_array());
                    uvs.push([
                        gx as f32 / terrain.columns as f32,
                        gz as f32 / terrain.rows as f32,
                    ]);
                }
            }
            for z in 0..rows {
                for x in 0..columns {
                    let a = z * (columns + 1) + x;
                    let b = a + 1;
                    let c = a + columns + 1;
                    let d = c + 1;
                    indices.extend_from_slice(&[a, c, b, b, c, d]);
                }
            }
            meshes.push(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_indices(Indices::U32(indices)),
            );
        }
    }
    meshes
}
