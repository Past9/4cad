mod curve;
mod knots;
mod surface;

use std::cmp::max;

use cgmath::{Matrix4, Point3, Transform, Vector3, Vector4, Zero};
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

fn basis(span: usize, u: f64, degree: usize, knots: &KnotVec) -> Vec<f64> {
    // Alg A2.2
    let mut basis_vals = vec![0.0; degree + 1];
    basis_vals[0] = 1.0;

    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;
        for r in 0..j {
            let temp = basis_vals[r] / (right[r + 1] + left[j - r]);
            basis_vals[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }

        basis_vals[j] = saved;
    }

    basis_vals
}

fn lu_solve(mat_a: Vec<Vec<f64>>, points: Vec<Pt4>) -> () {
    // call lu_decomposition
    // call forward_substitution
    // call backward_substitution

    let mut x = vec![Pt4::zero(); points.len()];

    let decomp = lu_decomposition(mat_a);

    for i in 0..points.len() {
        //for j in 0..
    }
}

#[derive(Debug)]
struct LUDecomp {
    upper: Vec<Vec<f64>>,
    lower: Vec<Vec<f64>>,
}

fn lu_decomposition(mat: Vec<Vec<f64>>) -> LUDecomp {
    // Validate that the matrix is square and non-empty
    if mat.len() == 0 {
        panic!("Matrix is empty");
    }

    if mat[0].len() != mat.len() {
        panic!("{}x{} matrix is not square", mat.len(), mat[0].len());
    }

    // Doolittle's method
    let mut mat_u = vec![vec![0.0; mat.len()]; mat.len()];
    let mut mat_l = mat_u.clone();

    for i in 0..mat.len() {
        for k in i..mat.len() {
            mat_u[i][k] = mat[i][k] - (0..i).map(|j| mat_l[i][j] * mat_u[j][k]).sum::<f64>();

            if i == k {
                mat_l[k][i] = 1.0;
            } else {
                mat_l[k][i] = mat[k][i] - (0..i).map(|j| mat_l[k][j] * mat_u[j][i]).sum::<f64>();
                if mat_u[i][i] != 0.0 {
                    mat_l[k][i] /= mat_u[i][i];
                } else {
                    mat_l[k][i] = 0.0;
                }
            }
        }
    }

    let output = LUDecomp {
        upper: mat_u,
        lower: mat_l,
    };

    output
}

fn forward_substitution(mat_l: &Vec<Vec<f64>>, mat_b: Vec<f64>) -> Vec<f64> {
    let q = mat_b.len();
    let mut mat_y = vec![0.0; q];
    mat_y[0] = mat_b[0] / mat_l[0][0];

    for i in 1..q {
        mat_y[i] = mat_b[i] - (0..i).map(|j| mat_l[i][j] * mat_y[j]).sum::<f64>();
        mat_y[i] /= mat_l[i][i];
    }

    mat_y
}

fn backward_substitution(mat_u: &Vec<Vec<f64>>, mat_y: Vec<f64>) -> Vec<f64> {
    let q = mat_y.len();
    let mut mat_x = vec![0.0; q];
    mat_x[q - 1] = mat_y[q - 1] / mat_u[q - 1][q - 1];
    for i in (0..=q - 2).rev() {
        mat_x[i] = mat_y[i] - (i..q).map(|j| mat_u[i][j] * mat_x[j]).sum::<f64>();
        mat_x[i] /= mat_u[i][i];
    }

    mat_x
}
