use std::iter::Sum;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};

use crate::{point3d::Point3D, rat, rational::Rat, HPoint};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Point4d {
    pub(crate) w: Rat,
    pub(crate) x: Rat,
    pub(crate) y: Rat,
    pub(crate) z: Rat,
}
impl Point4d {
    pub fn new(w: Rat, x: Rat, y: Rat, z: Rat) -> Self {
        Self { w, x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            w: 0.into(),
            x: 0.into(),
            y: 0.into(),
            z: 0.into(),
        }
    }

    pub fn project(&self) -> Point3D {
        Point3D::new(&self.x / &self.w, &self.y / &self.w, &self.z / &self.w)
    }
}
impl From<HPoint> for Point4d {
    fn from(value: HPoint) -> Self {
        Self::new(
            value.w.into(),
            value.x.into(),
            value.y.into(),
            value.z.into(),
        )
    }
}
impl std::fmt::Display for Point4d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}, {}, {}, {})",
            self.w, self.x, self.y, self.z
        ))
    }
}
impl Sum for Point4d {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut point = Point4d::zero();
        for item in iter {
            point = point + item;
        }
        point
    }
}

impl_op_ex!(+ |a: &Point4d, b: &Point4d| -> Point4d {
    Point4d::new(
        &a.w + &b.w,
        &a.x + &b.x,
        &a.y + &b.y,
        &a.z + &b.z,
    )
});

impl_op_ex_commutative!(*|a: &Point4d, b: &Rat| -> Point4d {
    Point4d {
        w: &a.w * b,
        x: &a.x * b,
        y: &a.y * b,
        z: &a.z * b,
    }
});
