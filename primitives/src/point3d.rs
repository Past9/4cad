use cgmath::{Point3, Vector3};

#[derive(PartialEq, Debug, Clone)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Point3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
impl std::fmt::Display for Point3d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.x, self.y, self.z))
    }
}
impl From<Point3d> for Vector3<f32> {
    fn from(value: Point3d) -> Self {
        Vector3 {
            x: value.x as f32,
            y: value.y as f32,
            z: value.z as f32,
        }
    }
}
impl From<Point3d> for Point3<f32> {
    fn from(value: Point3d) -> Self {
        Point3 {
            x: value.x as f32,
            y: value.y as f32,
            z: value.z as f32,
        }
    }
}
