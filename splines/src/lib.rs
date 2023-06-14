mod curve;
mod surface;

use cgmath::{Matrix4, Point3, Transform, Vector3, Vector4};
pub use curve::*;
pub use surface::*;

pub type Mat4 = Matrix4<f64>;
pub type Vec3 = Vector3<f64>;
pub type Pt3 = Point3<f64>;
pub type Pt4 = Vector4<f64>;

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

fn normalize_knots(knots: Vec<f64>) -> Vec<f64> {
    let max_knot = knots[knots.len() - 1].clone();
    knots.into_iter().map(|knot| knot / &max_knot).collect()
}

fn basis(knots: &[f64], j: usize, m: usize, t: f64) -> f64 {
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

fn knot_span(knots: &[f64], num_pts: usize, pos: f64) -> usize {
    let degree = knots.len() - num_pts - 1;

    if pos == knots[num_pts] {
        return num_pts - 1;
    }

    let mut low = degree;
    let mut high = num_pts + 1;
    let mut mid = (low + high) / 2;

    while pos < knots[mid] || pos >= knots[mid + 1] {
        if pos < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }

        mid = (low + high) / 2;
    }

    return mid;
}
