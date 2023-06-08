use cgmath::{vec3, vec4, Vector3, Vector4};
use crevice::std140::AsStd140;

pub mod camera;
pub mod lights;
pub mod model;
pub mod renderer;
pub mod scene;

pub struct PixelViewport {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba([f32; 4]);
impl Rgba {
    pub const RED: Self = Self([1.0, 0.0, 0.0, 1.0]);
    pub const GREEN: Self = Self([0.0, 1.0, 0.0, 1.0]);
    pub const BLUE: Self = Self([0.0, 0.0, 1.0, 1.0]);

    pub const DARK_RED: Self = Self([0.5, 0.0, 0.0, 1.0]);
    pub const DARK_GREEN: Self = Self([0.0, 0.5, 0.0, 1.0]);
    pub const DARK_BLUE: Self = Self([0.0, 0.0, 0.5, 1.0]);

    pub const YELLOW: Self = Self([1.0, 1.0, 0.0, 1.0]);
    pub const MAGENTA: Self = Self([1.0, 0.0, 1.0, 1.0]);
    pub const CYAN: Self = Self([0.0, 1.0, 1.0, 1.0]);

    pub const BLACK: Self = Self([0.0, 0.0, 0.0, 1.0]);
    pub const WHITE: Self = Self([1.0, 1.0, 1.0, 1.0]);

    pub const TRANSPARENT: Self = Self([0.0, 0.0, 0.0, 0.0]);

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }

    pub fn rgb(&self) -> Rgb {
        Rgb::new(self.r(), self.g(), self.b())
    }

    pub fn r(&self) -> f32 {
        self.0[0]
    }

    pub fn g(&self) -> f32 {
        self.0[1]
    }

    pub fn b(&self) -> f32 {
        self.0[2]
    }

    pub fn a(&self) -> f32 {
        self.0[3]
    }

    pub fn from_vec(vec: Vector4<f32>) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }

    pub fn to_vec(&self) -> Vector4<f32> {
        vec4(self.0[0], self.0[1], self.0[2], self.0[3])
    }

    pub fn to_floats(&self) -> [f32; 4] {
        self.0
    }

    pub fn set_r(&mut self, r: f32) {
        self.0[0] = r;
    }

    pub fn set_g(&mut self, g: f32) {
        self.0[1] = g;
    }

    pub fn set_b(&mut self, b: f32) {
        self.0[2] = b;
    }

    pub fn set_a(&mut self, a: f32) {
        self.0[3] = a;
    }
}
impl AsStd140 for Rgba {
    type Output = crevice::std140::Vec4;

    fn as_std140(&self) -> Self::Output {
        self.to_vec().as_std140()
    }

    fn from_std140(val: Self::Output) -> Self {
        Self::from_vec(Vector4::from_std140(val))
    }
}

pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    Rgba::new(r, g, b, a)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb([f32; 3]);
impl Rgb {
    pub const RED: Self = Self([1.0, 0.0, 0.0]);
    pub const GREEN: Self = Self([0.0, 1.0, 0.0]);
    pub const BLUE: Self = Self([0.0, 0.0, 1.0]);

    pub const YELLOW: Self = Self([1.0, 1.0, 0.0]);
    pub const MAGENTA: Self = Self([1.0, 0.0, 1.0]);
    pub const CYAN: Self = Self([0.0, 1.0, 1.0]);

    pub const BLACK: Self = Self([0.0, 0.0, 0.0]);
    pub const WHITE: Self = Self([1.0, 1.0, 1.0]);

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b])
    }

    pub fn r(&self) -> f32 {
        self.0[0]
    }

    pub fn g(&self) -> f32 {
        self.0[1]
    }

    pub fn b(&self) -> f32 {
        self.0[2]
    }

    pub fn from_vec(vec: Vector3<f32>) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }

    pub fn to_vec(&self) -> Vector3<f32> {
        vec3(self.0[0], self.0[1], self.0[2])
    }

    pub fn to_floats(&self) -> [f32; 3] {
        self.0
    }

    pub fn set_r(&mut self, r: f32) {
        self.0[0] = r;
    }

    pub fn set_g(&mut self, g: f32) {
        self.0[1] = g;
    }

    pub fn set_b(&mut self, b: f32) {
        self.0[2] = b;
    }
}
impl AsStd140 for Rgb {
    type Output = crevice::std140::Vec3;

    fn as_std140(&self) -> Self::Output {
        self.to_vec().as_std140()
    }

    fn from_std140(val: Self::Output) -> Self {
        Self::from_vec(Vector3::from_std140(val))
    }
}

pub fn rgb(r: f32, g: f32, b: f32) -> Rgb {
    Rgb::new(r, g, b)
}
