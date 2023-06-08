use crate::lights::{LightBuffers, Lights};
use crate::{
    camera::Camera,
    model::{Geometry, GeometryBuffers},
    Rgba,
};
use cgmath::{vec3, Matrix4, Quaternion, Vector3, Zero};
use vulkano::memory::allocator::MemoryAllocator;

pub struct SceneBuilder {
    camera: Option<Camera>,
    background: Rgba,
    lights: Option<Lights>,
    geometry: Option<Geometry>,
}
impl SceneBuilder {
    pub fn empty() -> Self {
        Self {
            camera: None,
            background: Rgba::BLACK,
            lights: None,
            geometry: None,
        }
    }

    pub fn build(self) -> Scene {
        Scene {
            background: self.background.clone(),
            orientation: Orientation::zero(),
            lights: self.lights.unwrap_or(Lights::new()),
            camera: self.camera.expect("No camera"),
            geometry: self.geometry.unwrap_or(Geometry::new()),
        }
    }

    pub fn camera(&mut self, camera: Camera) -> &mut Self {
        self.camera = Some(camera);
        self
    }

    pub fn background(&mut self, background: Rgba) -> &mut Self {
        self.background = background;
        self
    }

    pub fn lights(&mut self, lights: Lights) -> &mut Self {
        self.lights = Some(lights);
        self
    }

    pub fn geometry(&mut self, geometry: Geometry) -> &mut Self {
        self.geometry = Some(geometry);
        self
    }
}

pub struct Scene {
    background: Rgba,
    camera: Camera,
    orientation: Orientation,
    lights: Lights,
    geometry: Geometry,
}
impl Scene {
    pub fn background(&self) -> Rgba {
        self.background
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn orientation(&self) -> &Orientation {
        &self.orientation
    }

    pub fn orientation_mut(&mut self) -> &mut Orientation {
        &mut self.orientation
    }

    pub fn geometry_buffers(&self, allocator: &(impl MemoryAllocator + ?Sized)) -> GeometryBuffers {
        self.geometry.build_buffers(allocator)
    }

    pub fn light_buffers(&self, allocator: &(impl MemoryAllocator + ?Sized)) -> LightBuffers {
        self.lights.build_buffers(allocator)
    }
}

#[derive(Clone, Debug)]
pub struct Orientation {
    offset: Vector3<f32>,
    rotation: Quaternion<f32>,
}
impl Orientation {
    pub fn zero() -> Self {
        Self {
            offset: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::zero(),
        }
    }

    pub fn set_offset(&mut self, offset: Vector3<f32>) {
        self.offset = offset;
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.offset) * Matrix4::from(self.rotation)
    }
}
