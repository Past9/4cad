use crate::Rgb;
use cgmath::{vec3, Vector3};
use crevice::std140::AsStd140;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage},
};

#[derive(AsStd140, Clone, Debug)]
pub struct DirectionalLight {
    direction: Vector3<f32>,
    color: Rgb,
    intensity: f32,
}
impl DirectionalLight {
    pub fn new(direction: Vector3<f32>, color: Rgb, intensity: f32) -> Self {
        Self {
            direction,
            color,
            intensity,
        }
    }

    pub fn zero() -> Self {
        Self {
            direction: vec3(0.0, 0.0, 1.0),
            color: Rgb::BLACK,
            intensity: 0.0,
        }
    }

    pub fn buffer(
        allocator: &(impl MemoryAllocator + ?Sized),
        lights: Vec<DirectionalLight>,
    ) -> Subbuffer<[Std140DirectionalLight]> {
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
