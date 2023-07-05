use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use primitives::Vec3;
use splines::SurfacePoint;
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::Rgba;

use super::{MaterialId, ModelObjectId};

#[derive(Clone, Debug)]
pub struct ModelSurface {
    id: ModelObjectId,
    vertices: Vec<SurfaceVertex>,
    indices: Vec<u32>,
    material_id: MaterialId,
}
impl ModelSurface {
    pub fn new(
        id: ModelObjectId,
        vertices: Vec<SurfaceVertex>,
        indices: Vec<u32>,
        material_id: MaterialId,
    ) -> Self {
        Self {
            id,
            vertices,
            indices,
            material_id,
        }
    }

    pub fn vertices(&self) -> &[SurfaceVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn is_opaque(&self) -> bool {
        self.material_id.is_opaque()
    }

    pub fn is_translucent(&self) -> bool {
        self.material_id.is_translucent()
    }

    pub fn material_id(&self) -> MaterialId {
        self.material_id
    }

    pub fn from_surface_points(points: Vec<Vec<SurfacePoint>>, material: MaterialId) -> Self {
        let mut indices = Vec::new();
        let segments_u = points.len() - 1;
        let segments_v = points[0].len() - 1;
        for u in 1..=segments_u as u32 {
            for v in 1..=segments_v as u32 {
                // Quad corners
                let bl = index(u - 1, v - 1, segments_u as u32); // Bottom left
                let br = index(u, v - 1, segments_u as u32); // Bottom right
                let tl = index(u - 1, v, segments_u as u32); // Top left
                let tr = index(u, v, segments_u as u32); // Top right

                // Triangle 1
                indices.push(bl);
                indices.push(br);
                indices.push(tr);

                // Triangle 2
                indices.push(bl);
                indices.push(tr);
                indices.push(tl);
            }
        }

        ModelSurface::new(
            0.into(),
            points
                .into_iter()
                .flat_map(|row| {
                    row.into_iter().map(|point| SurfaceVertex {
                        position: [
                            point.position.x as f32,
                            point.position.y as f32,
                            point.position.z as f32,
                        ],
                        normal: [
                            point.normal.x as f32,
                            point.normal.y as f32,
                            point.normal.z as f32,
                        ],
                    })
                })
                .collect(),
            indices,
            material,
        )
    }
}

fn index(u: u32, v: u32, segments_u: u32) -> u32 {
    u * (segments_u + 1) + v
}

#[derive(Default, Debug, Copy, Clone)]
pub struct SurfaceVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
impl SurfaceVertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Vertex, Zeroable, Pod)]
pub struct BufferedSurfaceVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    normal: [f32; 3],
    #[format(R32_UINT)]
    material_idx: u32,
}
impl BufferedSurfaceVertex {
    pub fn new(vertex: &SurfaceVertex, material_id: MaterialId) -> Self {
        Self {
            position: vertex.position.clone(),
            normal: vertex.normal.clone(),
            material_idx: material_id.index(),
        }
    }
}
