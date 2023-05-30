use crate::Vec3;

pub mod model;

pub struct Triangle {
    pub vertex_a: Vec3,
    pub vertex_b: Vec3,
    pub vertex_c: Vec3,

    pub normal_a: Vec3,
    pub normal_b: Vec3,
    pub normal_c: Vec3,

    pub edge_ab: bool,
    pub edge_bc: bool,
    pub edge_ca: bool,
}

pub struct Line {
    pub vertex_a: Vec3,
    pub vertex_b: Vec3,

    pub expand_a: Vec3,
    pub expand_b: Vec3,
}

pub struct Point {
    pub vertex: Vec3,
    pub expand: Vec3,
}

pub trait Mesh {
    fn triangles(&self) -> &[Triangle];
    fn lines(&self) -> &[Line];
    fn points(&self) -> &[Point];
}

#[allow(dead_code)]
pub(crate) struct FloatRange {
    num_increments: usize,
    start: f32,
    increment: f32,
    count: usize,
}
impl FloatRange {
    #[allow(dead_code)]
    pub fn new(lower_bound: f32, upper_bound: f32, num_increments: usize) -> Self {
        let increment = (upper_bound - lower_bound) / num_increments as f32;
        Self {
            num_increments,
            start: lower_bound,
            increment,
            count: 0,
        }
    }
}
impl Iterator for FloatRange {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.num_increments + 1 {
            let next = self.start + self.increment * self.count as f32;
            self.count += 1;
            Some(next)
        } else {
            None
        }
    }
}
