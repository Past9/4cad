use std::iter::{Product, Sum};

use auto_ops::{impl_op_ex, impl_op_ex_commutative};

use crate::point3d::Point3d;

#[derive(PartialEq, Debug, Clone)]
pub struct Point4d {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Point4d {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    pub fn new_ints(w: i64, x: i64, y: i64, z: i64) -> Self {
        Self {
            w: w as f64,
            x: x as f64,
            y: y as f64,
            z: z as f64,
        }
    }

    pub fn zero() -> Self {
        Self {
            w: 0.into(),
            x: 0.into(),
            y: 0.into(),
            z: 0.into(),
        }
    }

    pub fn project(&self) -> Point3d {
        Point3d::new(self.x / self.w, self.y / self.w, self.z / self.w)
    }

    pub fn truncate(&self) -> Point3d {
        Point3d::new(self.x, self.y, self.z)
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
        a.w + b.w,
        a.x + b.x,
        a.y + b.y,
        a.z + b.z,
    )
});

/*
impl_op_ex_commutative!(*|a: &Point4d, b: f64| -> Point4d {
    Point4d {
        w: a.w * b,
        x: a.x * b,
        y: a.y * b,
        z: a.z * b,
    }
});

*/
