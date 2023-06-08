use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};

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
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct BufferedSurfaceVertex {
    position: [f32; 3],
    normal: [f32; 3],
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
vulkano::impl_vertex!(BufferedSurfaceVertex, position, normal, material_idx);
