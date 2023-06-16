mod curve;
mod knots;
mod surface;

use std::cmp::max;

use cgmath::{Matrix4, Point3, Transform, Vector3, Vector4};
pub use curve::*;
pub use knots::KnotVec;
pub use surface::*;

pub type Mat4 = Matrix4<f64>;
pub type Vec3 = Vector3<f64>;
pub type Pt3 = Point3<f64>;
pub type Pt4 = Vector4<f64>;

const TOL: f64 = 10.0e-8;

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

pub trait EPoint {
    fn as_f32(self) -> Point3<f32>;
    fn to_hpoint(self, w: f64) -> Pt4;
}
impl EPoint for Pt3 {
    fn as_f32(self) -> Point3<f32> {
        self.cast::<f32>().unwrap()
    }

    fn to_hpoint(self, w: f64) -> Pt4 {
        Pt4::new(self.x, self.y, self.z, w)
    }
}

pub trait HPoint {
    fn project(&self) -> Pt3;
    fn transform(&self, transform: &Matrix4<f64>) -> Self;
    fn weight(&self) -> Self;
    fn unweight(&self) -> Self;
}
impl HPoint for Pt4 {
    fn project(&self) -> Pt3 {
        Pt3 {
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

const BINOMIAL_COEFFICIENTS: [[f64; 10]; 10] = [
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 3.0, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 4.0, 6.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 5.0, 10.0, 10.0, 5.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 6.0, 15.0, 20.0, 15.0, 6.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 7.0, 21.0, 35.0, 35.0, 21.0, 7.0, 1.0, 0.0, 0.0],
    [1.0, 8.0, 28.0, 56.0, 70.0, 56.0, 28.0, 8.0, 1.0, 0.0],
    [1.0, 9.0, 36.0, 84.0, 126.0, 126.0, 84.0, 36.0, 9.0, 1.0],
];

/// Computes the binomial coefficient for (k, i)
fn bin(k: usize, i: usize) -> f64 {
    BINOMIAL_COEFFICIENTS[k][i]
}

fn basis(knots: &KnotVec, j: usize, m: usize, t: f64) -> f64 {
    if j == 0 && t == 0.0 {
        return 1.into();
    }

    if j == knots.len() - m - 1 && t == 1.0 {
        return 1.into();
    }

    let tj = knots[j];
    let tj1 = knots[j + 1];

    if m == 1 {
        if tj <= t && t < tj1 {
            1.0
        } else {
            0.0
        }
    } else {
        let tjm = &knots[j + m];
        let tjmsub1 = &knots[j + m - 1];

        let den1 = tjmsub1 - tj;
        let l = if den1 == 0.0 {
            0.into()
        } else {
            ((t - tj) / den1) * basis(knots, j, m - 1, t)
        };

        let den2 = tjm - tj1;
        let r = if den2 == 0.0 {
            0.into()
        } else {
            ((tjm - t) / den2) * basis(knots, j + 1, m - 1, t)
        };

        l + r
    }
}
