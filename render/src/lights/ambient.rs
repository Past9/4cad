use crate::Rgb;
use crevice::std140::AsStd140;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage},
};

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
    ) -> Subbuffer<[Std140AmbientLight]> {
        Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
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
