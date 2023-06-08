use crate::Rgb;
use cgmath::{point3, Point3};
use crevice::std140::AsStd140;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage},
};

#[derive(AsStd140, Clone, Debug)]
pub struct PointLight {
    position: Point3<f32>,
    color: Rgb,
    intensity: f32,
}
impl PointLight {
    pub fn new(position: Point3<f32>, color: Rgb, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }

    pub fn zero() -> Self {
        Self {
            position: point3(0.0, 0.0, 0.0),
            color: Rgb::BLACK,
            intensity: 0.0,
        }
    }

    pub fn buffer(
        allocator: &(impl MemoryAllocator + ?Sized),
        lights: Vec<PointLight>,
    ) -> Subbuffer<[Std140PointLight]> {
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
