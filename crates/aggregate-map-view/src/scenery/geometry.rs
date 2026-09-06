use aggregate_geography::Heightfield;
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

#[derive(Default)]
pub struct SceneryGeometry {
    pub positions: Vec<[f32;3]>,
    pub normals: Vec<[f32;3]>,
    pub uvs: Vec<[f32;2]>,
    pub colors: Vec<[f32;4]>,
}

impl SceneryGeometry {
    pub fn triangle(&mut self, points: [Vec3;3], uv: [Vec2;3], color: [f32;4]) {
        let normal = (points[1]-points[0]).cross(points[2]-points[0]).normalize_or_zero();
        for index in 0..3 {
            self.positions.push(points[index].to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uv[index].to_array());
            self.colors.push(color);
        }
    }

    pub fn into_mesh(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL,self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR,self.colors)
    }
}

/// Match the render mesh's two triangles per cell, rather than floating above a bilinear surface.
pub fn ground(terrain: &Heightfield, position: Vec2) -> Vec3 {
    let grid = position / terrain.size * Vec2::new(terrain.columns as f32,terrain.rows as f32);
    let x = (grid.x.floor() as u32).min(terrain.columns-1);
    let z = (grid.y.floor() as u32).min(terrain.rows-1);
    let fraction = (grid-Vec2::new(x as f32,z as f32)).clamp(Vec2::ZERO,Vec2::ONE);
    let a = terrain.position(x,z).y;
    let b = terrain.position(x+1,z).y;
    let c = terrain.position(x,z+1).y;
    let d = terrain.position(x+1,z+1).y;
    let height = if fraction.x+fraction.y <= 1. {
        a+(b-a)*fraction.x+(c-a)*fraction.y
    } else {
        d+(c-d)*(1.-fraction.x)+(b-d)*(1.-fraction.y)
    };
    Vec3::new(position.x,height,position.y)
}
