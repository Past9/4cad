use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::Rgba;

use super::ModelObjectId;

#[derive(Clone, Debug)]
pub struct ModelEdge {
    id: ModelObjectId,
    vertices: Vec<EdgeVertex>,
    color: Rgba,
}
impl ModelEdge {
    pub fn new(id: ModelObjectId, vertices: Vec<EdgeVertex>, color: Rgba) -> Self {
        Self {
            id,
            vertices,
            color,
        }
    }

    pub fn vertices(&self) -> &[EdgeVertex] {
        &self.vertices
    }

    pub fn color(&self) -> Rgba {
        self.color
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct EdgeVertex {
    pub position: [f32; 3],
    pub expand: [f32; 3],
}
impl EdgeVertex {
    pub fn new(position: Point3<f32>, expand: Vector3<f32>) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            expand: [expand.x, expand.y, expand.z],
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Vertex, Zeroable, Pod)]
pub struct BufferedEdgeVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    expand: [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}
impl BufferedEdgeVertex {
    pub fn new(vertex: &EdgeVertex, color: Rgba) -> Self {
        Self {
            position: vertex.position.clone(),
            expand: vertex.expand.clone(),
            color: color.to_floats(),
        }
    }
}
