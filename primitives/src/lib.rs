mod angle;

pub use angle::*;
use cgmath::{Matrix4, Point3, Transform, Vector3, Vector4};

pub const TOL: f64 = 10.0e-8;

pub type Mat4 = Matrix4<f64>;
pub type Vec3 = Vector3<f64>;
pub type Vec4 = Vector4<f64>;

pub trait TolEq {
    fn toleq(self, rhs: Self) -> bool;
    fn toleq_avg(self, rhs: Self) -> Option<Self>
    where
        Self: Sized;
}
impl TolEq for f64 {
    fn toleq(self, rhs: Self) -> bool {
        (self - rhs).abs() <= TOL
    }

    fn toleq_avg(self, rhs: Self) -> Option<Self> {
        if self.toleq(rhs) {
            Some((self + rhs) / 2.0)
        } else {
            None
        }
    }
}
impl TolEq for Vec3 {
    fn toleq(self, rhs: Self) -> bool {
        self.x.toleq(rhs.x) && self.y.toleq(rhs.y) && self.z.toleq(rhs.z)
    }

    fn toleq_avg(self, rhs: Self) -> Option<Self>
    where
        Self: Sized,
    {
        if self.toleq(rhs) {
            Some((self + rhs) / 2.0)
        } else {
            None
        }
    }
}
impl TolEq for Vec4 {
    fn toleq(self, rhs: Self) -> bool {
        self.x.toleq(rhs.x) && self.y.toleq(rhs.y) && self.z.toleq(rhs.z) && self.w.toleq(rhs.w)
    }

    fn toleq_avg(self, rhs: Self) -> Option<Self>
    where
        Self: Sized,
    {
        if self.toleq(rhs) {
            Some((self + rhs) / 2.0)
        } else {
            None
        }
    }
}
impl TolEq for Vec<f64> {
    fn toleq(self, rhs: Self) -> bool {
        if self.len() == rhs.len() {
            self.iter().zip(rhs.iter()).all(|(l, r)| l.toleq(*r))
        } else {
            false
        }
    }

    fn toleq_avg(self, rhs: Self) -> Option<Self>
    where
        Self: Sized,
    {
        if self.len() == rhs.len() {
            let mut knots = vec![];
            for i in 0..self.len() {
                let l = self[i];
                let r = rhs[i];

                if let Some(avg) = l.toleq_avg(r) {
                    knots.push(avg);
                } else {
                    return None;
                }
            }
            Some(knots)
        } else {
            None
        }
    }
}

pub trait EVec {
    fn as_f32(self) -> Vector3<f32>;
    fn to_hpoint(self, w: f64) -> Vec4;
}
impl EVec for Vec3 {
    fn as_f32(self) -> Vector3<f32> {
        self.cast::<f32>().unwrap()
    }

    fn to_hpoint(self, w: f64) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

pub trait HVec {
    fn project(&self) -> Vec3;
    fn transform(&self, transform: &Matrix4<f64>) -> Self;
    fn weight(&self) -> Self;
    fn unweight(&self) -> Self;
}
impl HVec for Vec4 {
    fn project(&self) -> Vec3 {
        Vec3 {
            x: self.x / self.w,
            y: self.y / self.w,
            z: self.z / self.w,
        }
    }

    fn transform(&self, transform: &Matrix4<f64>) -> Self {
        let xyz = Point3::new(self.x, self.y, self.z);
        let xyz = transform.transform_point(xyz);

        Self {
            x: xyz.x,
            y: xyz.y,
            z: xyz.z,
            w: self.w,
        }
    }

    fn weight(&self) -> Self {
        Self {
            x: self.x * self.w,
            y: self.y * self.w,
            z: self.z * self.w,
            w: self.w,
        }
    }

    fn unweight(&self) -> Self {
        Self {
            x: self.x / self.w,
            y: self.y / self.w,
            z: self.z / self.w,
            w: self.w,
        }
    }
}
