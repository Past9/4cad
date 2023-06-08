use std::sync::Arc;

use crevice::std140::AsStd140;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    memory::allocator::MemoryAllocator,
};

use crate::Rgb;

#[derive(AsStd140, Clone, Debug)]
pub struct AmbientLight {
    color: Rgb,
    intensity: f32,
}
impl AmbientLight {
    pub fn new(color: Rgb, intensity: f32) -> Self {
        Self { color, intensity }
    }

    pub fn zero() -> Self {
        Self {
            color: Rgb::BLACK,
            intensity: 0.0,
        }
    }

    pub fn buffer(
        allocator: &(impl MemoryAllocator + ?Sized),
        lights: Vec<AmbientLight>,
    ) -> Arc<CpuAccessibleBuffer<[Std140AmbientLight]>> {
        CpuAccessibleBuffer::from_iter(
            allocator,
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::default()
            },
            false,
            match lights.len() {
                len if len > 0 => lights,
                _ => vec![Self::zero()],
            }
            .into_iter()
            .map(|light| light.as_std140()),
        )
        .unwrap()
    }
}
