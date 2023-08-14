mod curve;
mod knots;
mod surface;

use std::cmp::min;

use cgmath::{InnerSpace, Zero};

pub use curve::*;
pub use knots::KnotVec;
use primitives::{TolEq, Vec3, Vec4};
pub use surface::*;

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

fn curve_derivatives(
    u: f64,
    weighted: &[Vec4],
    degree: usize,
    knots: &KnotVec,
    num_derivatives: usize,
) -> Vec<Vec4> {
    let num_derivatives = min(num_derivatives, degree);
    let mut derivatives = vec![Vec4::zero(); num_derivatives + 1];

    let span = knots.find_span(degree, u);
    let basis_derivatives = basis_derivatives(span, u, degree, &knots, num_derivatives);

    for k in 0..=num_derivatives {
        for j in 0..=degree {
            derivatives[k] += weighted[span - degree + j] * basis_derivatives[k][j];
        }
    }

    derivatives
}

fn nurbs_to_beziers(control_points: &[Vec4], degree: usize, knots: &KnotVec) -> Vec<Vec<Vec4>> {
    let n = control_points.len() - 1;
    let m = n + degree + 1;
    let mut a = degree;
    let mut b = degree + 1;
    let mut nb = 0;

    let new_bezier_points = vec![Vec4::zero(); degree + 1];
    let mut bezier_ctrl_pts: Vec<Vec<Vec4>> = Vec::new();

    bezier_ctrl_pts.push(new_bezier_points.clone());

    for i in 0..=degree {
        bezier_ctrl_pts[nb][i] = control_points[i];
    }

    while b < m {
        let i = b;
        while b < m && knots[b + 1] == knots[b] {
            b += 1;
        }

        let mult = b - i + 1;
        if mult < degree {
            let numer = knots[b] - knots[a];
            let mut alphas = vec![0.0; degree - mult];
            for j in ((mult + 1)..=degree).rev() {
                alphas[j - mult - 1] = numer / (knots[a + j] - knots[a]);
            }

            let r = degree - mult;
            for j in 1..=r {
                let save = r - j;
                let s = mult + j;
                for k in (s..=degree).rev() {
                    let alpha = alphas[k - s];
                    bezier_ctrl_pts[nb][k] =
                        bezier_ctrl_pts[nb][k] * alpha + bezier_ctrl_pts[nb][k - 1] * (1.0 - alpha);
                }

                if b < m {
                    if bezier_ctrl_pts.len() - 1 < nb + 1 {
                        bezier_ctrl_pts.push(new_bezier_points.clone());
                    }
                    bezier_ctrl_pts[nb + 1][save] = bezier_ctrl_pts[nb][degree];
                }
            }
        }

        nb += 1;

        if b < m {
            for i in (degree - mult)..=degree {
                if bezier_ctrl_pts.len() - 1 < nb {
                    bezier_ctrl_pts.push(new_bezier_points.clone());
                }
                bezier_ctrl_pts[nb][i] = control_points[b - degree + i];
            }
            a = b;
            b += 1;
        }
    }

    bezier_ctrl_pts
}

/// Implements A3.6
fn surface_derivatives(
    u: f64,
    v: f64,
    weighted: &[Vec<Vec4>],
    degree_u: usize,
    degree_v: usize,
    knots_u: &KnotVec,
    knots_v: &KnotVec,
    num_derivatives: usize,
) -> Vec<Vec<Vec4>> {
    let num_derivatives_u = usize::min(num_derivatives, degree_u);
    let num_derivatives_v = usize::min(num_derivatives, degree_v);
    let mut derivatives = vec![vec![Vec4::zero(); num_derivatives + 1]; num_derivatives + 1];

    for k in (degree_u + 1)..=num_derivatives {
        for l in 0..=(num_derivatives - k) {
            derivatives[k][l] = Vec4::zero();
        }
    }

    for l in (degree_v + 1)..=num_derivatives {
        for k in 0..=(num_derivatives - l) {
            derivatives[k][l] = Vec4::zero();
        }
    }

    let span_u = knots_u.find_span(degree_u, u);
    let basis_derivatives_u = basis_derivatives(span_u, u, degree_u, knots_u, num_derivatives_u);

    let span_v = knots_v.find_span(degree_v, v);
    let basis_derivatives_v = basis_derivatives(span_v, v, degree_v, knots_v, num_derivatives_v);

    let mut temp = vec![Vec4::zero(); degree_v + 1];

    for k in 0..=num_derivatives_u {
        for s in 0..=degree_v {
            temp[s] = Vec4::zero();
            for r in 0..=degree_u {
                temp[s] += basis_derivatives_u[k][r]
                    * weighted[span_u + r - degree_u][span_v + s - degree_v]
            }
        }
        let dd = usize::min(num_derivatives - k, num_derivatives_v);
        for l in 0..=dd {
            derivatives[k][l] = Vec4::zero();
            for s in 0..=degree_v {
                derivatives[k][l] += basis_derivatives_v[l][s] * temp[s];
            }
        }
    }

    derivatives
}

/// Evaluates the point at `u` plus the specified number of derivaties an returns a
/// `(num_derivatives + 1) x (degree)`-dimensional `Vec<Vec<f64>>`. When referencing
/// this vector, the first index is the i-th derivative (with `0` being the 0-th
/// derivative, or simply the point on the curve at `u`), and the second is the
/// index of the basis function that was evaluated.
fn basis_derivatives(
    span: usize,
    u: f64,
    degree: usize,
    knots: &KnotVec,
    num_derivatives: usize,
) -> Vec<Vec<f64>> {
    let mut left = vec![1.0; degree + 1];
    let mut right = vec![1.0; degree + 1];
    let mut ndu = vec![vec![1.0; degree + 1]; degree + 1];

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            // Lower triangle
            ndu[j][r] = right[r + 1] + left[j - r];
            let temp = ndu[r][j - 1] / ndu[j][r];

            // Upper triangle
            ndu[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        ndu[j][j] = saved;
    }

    let mut derivatives: Vec<Vec<f64>> =
        vec![vec![0.0; degree + 1]; usize::min(degree, num_derivatives) + 1];

    // Load the basis functions
    for j in 0..=degree {
        derivatives[0][j] = ndu[j][degree];
    }

    // Begin calculating derivatives
    let mut a: Vec<Vec<f64>> = vec![vec![1.0; degree + 1]; 2];

    // This section computes the derivatives.
    // Loop over the function index
    for r in 0..=degree {
        // Alternate rows in array a
        let mut s1 = 0;
        let mut s2 = 1;

        a[0][0] = 1.0;

        // Loop to compute kth derivative
        for k in 1..=num_derivatives {
            let mut d = 0.0;

            let rk = r as i32 - k as i32;
            let pk = degree as i32 - k as i32;

            if r >= k {
                a[s2][0] = a[s1][0] / ndu[(pk + 1) as usize][rk as usize];
                d = a[s2][0] * ndu[rk as usize][pk as usize];
            }

            let j1 = if rk >= -1 { 1 } else { -rk } as usize;

            let j2 = if r as i32 - 1 <= pk {
                k - 1
            } else {
                degree - r
            };

            for j in j1..=j2 {
                a[s2][j] =
                    (a[s1][j] - a[s1][j - 1]) / ndu[(pk + 1) as usize][(rk + j as i32) as usize];
                d += a[s2][j] * ndu[(rk + j as i32) as usize][pk as usize];
            }

            if r <= pk as usize {
                a[s2][k] = -a[s1][k - 1] / ndu[(pk + 1) as usize][r];
                d += a[s2][k] * ndu[r][pk as usize];
            }

            derivatives[k][r] = d;

            // Switch rows
            let temp = s1;
            s1 = s2;
            s2 = temp;
        }
    }

    // Multiply through by the correct factors
    let mut r = degree as f64;
    for k in 1..=num_derivatives {
        for j in 0..=degree {
            derivatives[k][j] *= r;
        }
        r *= degree as f64 - k as f64;
    }

    derivatives
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

fn parameterize_by_chord_len(points: &[Vec4]) -> Vec<f64> {
    let n = points.len();
    let mut chord_lens = vec![0.0; n + 1];
    chord_lens[n] = 1.0;
    for i in 1..n {
        let dist = (points[i] - points[i - 1]).magnitude();
        chord_lens[i] = dist.sqrt();
    }
    let total_chord_len: f64 = chord_lens.iter().skip(1).take(chord_lens.len() - 2).sum();

    let mut uk = vec![0.0; n];
    for i in 0..n {
        uk[i] = chord_lens.iter().take(i + 1).sum::<f64>() / total_chord_len;
    }

    uk
}

fn transpose<T: Clone>(grid: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut out = Vec::new();

    for i in 0..grid.len() {
        let mut row = Vec::new();
        for j in 0..grid[0].len() {
            row.push(grid[j][i].clone());
        }
        out.push(row);
    }

    out
}

/// Returns an arbitrary unit vector _b_ that is orthogonal to `a`. This is done by
/// solving the equation _a · b_ = 0 for _b_ and then normalizing _b_. When solving
/// the equation
///
/// _a<sub>i</sub>b<sub>i</sub>_ + _a<sub>j</sub>b<sub>j</sub>_ + _a<sub>k</sub>k<sub>k</sub>_ = 0
///
/// we set two of _b_'s components, _b<sub>i</sub>_ and _b<sub>j</sub>_ to 1 and then solve
/// for the remaining _b<sub>k</sub>_, like this:
///
/// _b<sub>k</sub>_ = (-_a<sub>i</sub>_ - _a<sub>j</sub>_) / _a<sub>k</sub>_
///
/// Care must be taken to select _k_ so that _a<sub>k</sub>_ is not 0 to avoid division by 0.
/// If all components of _a_ are zero, an error is thrown because there is no vector that is
/// orthogonal to a zero-length vector.
fn arbitrary_orthonormal(a: Vec3) -> Vec3 {
    // Select k, the index of a's first non-zero component
    let mut k: Option<usize> = None;
    for n in 0..3 {
        if !a[n].toleq(0.0) {
            k = Some(n);
            break;
        }
    }

    let k = k.expect(&format!("Could not find a vector orthogonal to {:?}", a));

    // Formulate the numerator and denominator
    let mut num: f64 = 0.0;
    let mut den: f64 = 0.0;
    for n in 0..3 {
        if n == k {
            // The kth component of a is the denominator
            den = a[n];
        } else {
            // The numerator is the negative sum of all of
            // a's other components
            num -= a[n];
        }
    }

    // Create the non-normalized orthogonal vector b
    let mut b = Vec3::zero();
    for n in 0..3 {
        if n == k {
            // The kth component of b is num / den
            b[n] = num / den;
        } else {
            // All other components of b are 1
            b[n] = 1.0;
        }
    }

    // Normalize and return b
    b.normalize()
}

#[cfg(test)]
mod tests {
    use crate::transpose;

    #[test]
    fn transposes_grid() {
        assert_eq!(
            transpose(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]),
            vec![vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9],]
        );
    }
}
