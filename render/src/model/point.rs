use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};

use crate::Rgba;

use super::ModelObjectId;

#[derive(Debug, Clone)]
pub struct ModelPoint {
    id: ModelObjectId,
    position: [f32; 3],
    expand: [f32; 3],
    color: Rgba,
}
impl ModelPoint {
    pub fn new(
        id: ModelObjectId,
        position: Point3<f32>,
        expand: Vector3<f32>,
        color: Rgba,
    ) -> Self {
        Self {
            id,
            position: [position.x, position.y, position.z],
            expand: [expand.x, expand.y, expand.z],
            color,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct BufferedPointVertex {
    position: [f32; 3],
    expand: [f32; 3],
    color: [f32; 4],
}
impl BufferedPointVertex {
    pub fn new(vertex: &ModelPoint) -> Self {
        Self {
            position: vertex.position.clone(),
            expand: vertex.expand.clone(),
            color: vertex.color.to_floats(),
        }
    }
}
vulkano::impl_vertex!(BufferedPointVertex, position, expand, color);
