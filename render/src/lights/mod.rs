mod ambient;
mod directional;
mod point;

use std::sync::Arc;

pub use ambient::*;
use cgmath::{Point3, Vector3};
pub use directional::*;
pub use point::*;
use vulkano::{buffer::CpuAccessibleBuffer, memory::allocator::MemoryAllocator};

use crate::Rgb;

pub struct LightBuffers {
    pub ambient: Arc<CpuAccessibleBuffer<[Std140AmbientLight]>>,
    pub directional: Arc<CpuAccessibleBuffer<[Std140DirectionalLight]>>,
    pub point: Arc<CpuAccessibleBuffer<[Std140PointLight]>>,
}

#[derive(Clone)]
pub struct Lights {
    ambient: Vec<AmbientLight>,
    directional: Vec<DirectionalLight>,
    point: Vec<PointLight>,
}
impl Lights {
    pub fn new() -> Self {
        Self {
            ambient: vec![],
            directional: vec![],
            point: vec![],
        }
    }

    pub fn empty() -> Self {
        Self {
            ambient: vec![],
            directional: vec![],
            point: vec![],
        }
    }

    pub fn ambient(self, color: Rgb, intensity: f32) -> Self {
        let Self {
            mut ambient,
            directional,
            point,
        } = self;

        ambient.push(AmbientLight::new(color, intensity));

        Self {
            ambient,
            directional,
            point,
        }
    }

    pub fn point(self, position: Point3<f32>, color: Rgb, intensity: f32) -> Self {
        let Self {
            ambient,
            directional,
            mut point,
        } = self;

        point.push(PointLight::new(position, color, intensity));

        Self {
            ambient,
            directional,
            point,
        }
    }

    pub fn directional(self, direction: Vector3<f32>, color: Rgb, intensity: f32) -> Self {
        let Self {
            ambient,
            mut directional,
            point,
        } = self;

        directional.push(DirectionalLight::new(direction, color, intensity));

        Self {
            ambient,
            directional,
            point,
        }
    }

    pub fn build_buffers(&self, allocator: &(impl MemoryAllocator + ?Sized)) -> LightBuffers {
        LightBuffers {
            ambient: AmbientLight::buffer(allocator, self.ambient.clone()),
            directional: DirectionalLight::buffer(allocator, self.directional.clone()),
            point: PointLight::buffer(allocator, self.point.clone()),
        }
    }
}
