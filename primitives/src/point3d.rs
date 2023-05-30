use cgmath::Vector3;

use crate::{hpoint::HPoint, rational::Rat, Int};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Point3D {
    x: Rat,
    y: Rat,
    z: Rat,
}
impl Point3D {
    pub fn new(x: Rat, y: Rat, z: Rat) -> Self {
        Self { x, y, z }
    }

    pub fn new_ints(x: Int, y: Int, z: Int) -> Self {
        Self::new(x.into(), y.into(), z.into())
    }

    pub fn homogenize(self, w: Rat) -> HPoint {
        HPoint::from_rats(w, self.x, self.y, self.z)
    }

    pub fn homogenize_int(self, w: Int) -> HPoint {
        self.homogenize(w.into())
    }
}
impl std::fmt::Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.x, self.y, self.z))
    }
}
impl From<Point3D> for Vector3<f32> {
    fn from(value: Point3D) -> Self {
        Vector3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}
