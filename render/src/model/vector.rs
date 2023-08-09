use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::Rgba;

#[derive(Debug, Clone)]
pub struct ModelVector {
    origin: [f32; 3],
    direction: [f32; 3],
    color: Rgba,
}
impl ModelVector {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, color: Rgba) -> Self {
        Self {
            origin: [origin.x, origin.y, origin.z],
            direction: [direction.x, direction.y, direction.z],
            color,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Vertex, Zeroable, Pod)]
pub struct BufferedVectorVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}
impl BufferedVectorVertex {
    pub fn create_vertices(vector: &ModelVector) -> (Self, Self) {
        (
            // Start vertex
            Self {
                position: vector.origin,
                color: vector.color.to_floats(),
            },
            // End vertex
            Self {
                position: [
                    vector.origin[0] + vector.direction[0],
                    vector.origin[1] + vector.direction[1],
                    vector.origin[2] + vector.direction[2],
                ],
                color: vector.color.lighten(0.2).to_floats(),
            },
        )
    }
}
