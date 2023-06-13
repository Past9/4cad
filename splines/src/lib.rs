mod curve;
mod surface;

use cgmath::{Point3, Vector3, Vector4};
pub use curve::*;
pub use surface::*;

type Pt3 = Point3<f64>;
type Pt4 = Vector4<f64>;

pub trait SplineHelpers3 {
    fn as_f32(&self) -> Point3<f32>;
}
impl SplineHelpers3 for Pt3 {
    fn as_f32(&self) -> Point3<f32> {
        self.cast::<f32>().unwrap()
    }
}

pub trait SplineHelpers4 {
    fn project(&self) -> Pt3;
}
impl SplineHelpers4 for Pt4 {
    fn project(&self) -> Pt3 {
        Pt3 {
            x: self.x / self.w,
            y: self.y / self.w,
            z: self.z / self.w,
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
