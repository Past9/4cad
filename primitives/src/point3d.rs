use cgmath::{Point3, Vector3};

use crate::{rational::Rat, Int};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Point3d {
    x: Rat,
    y: Rat,
    z: Rat,
}
impl Point3d {
    pub fn new(x: Rat, y: Rat, z: Rat) -> Self {
        Self { x, y, z }
    }

    pub fn new_ints(x: Int, y: Int, z: Int) -> Self {
        Self::new(x.into(), y.into(), z.into())
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
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}
impl From<Point3d> for Point3<f32> {
    fn from(value: Point3d) -> Self {
        Point3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}
