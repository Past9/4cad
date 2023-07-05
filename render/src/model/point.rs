use bytemuck::{Pod, Zeroable};
use cgmath::Zero;
use primitives::Vec3;
use vulkano::pipeline::graphics::vertex_input::Vertex;

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
    pub fn new(id: ModelObjectId, position: Vec3, expand: Vec3, color: Rgba) -> Self {
        Self {
            id,
            position: [position.x as f32, position.y as f32, position.z as f32],
            expand: [expand.x as f32, expand.y as f32, expand.z as f32],
            color,
        }
    }

    pub fn from_vec3(point: Vec3, color: Rgba) -> Self {
        Self::new(0.into(), point, Vec3::zero(), color)
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Vertex, Zeroable, Pod)]
pub struct BufferedPointVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    expand: [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
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
