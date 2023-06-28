mod curve;
mod knots;
mod surface;

use std::cmp::min;

use cgmath::{InnerSpace, Zero};

pub use curve::*;
pub use knots::KnotVec;
use primitives::Vec4;
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

fn get_interpolation_params(points: &[Vec4]) -> Vec<f64> {
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
