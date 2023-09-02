mod builders;

use crate::{basis, bin, knots::KnotVec, surface_derivatives, Vec4};
use cgmath::{InnerSpace, Matrix4, Zero};

pub use builders::*;
use primitives::{EVec, HVec, TolEq, Vec3, TOL};

pub enum SurfaceDirection {
    U,
    V,
}

pub struct SurfacePoint {
    pub position: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone)]
pub struct Surface {
    unweighted: Vec<Vec<Vec4>>,
    weighted: Vec<Vec<Vec4>>,
    knots_u: KnotVec,
    knots_v: KnotVec,
    degree_u: usize,
    degree_v: usize,
    order_u: usize,
    order_v: usize,
}
impl Surface {
    pub fn create_unweighted(
        unweighted: Vec<Vec<Vec4>>,
        knots_u: KnotVec,
        knots_v: KnotVec,
    ) -> Self {
        let weighted: Vec<Vec<Vec4>> = unweighted
            .iter()
            .map(|row| row.iter().map(HVec::weight).collect())
            .collect();
        Self::create(unweighted, weighted, knots_u, knots_v)
    }

    pub fn create_weighted(weighted: Vec<Vec<Vec4>>, knots_u: KnotVec, knots_v: KnotVec) -> Self {
        let unweighted: Vec<Vec<Vec4>> = weighted
            .iter()
            .map(|row| row.iter().map(HVec::unweight).collect())
            .collect();
        Self::create(unweighted, weighted, knots_u, knots_v)
    }

    pub fn create(
        unweighted: Vec<Vec<Vec4>>,
        weighted: Vec<Vec<Vec4>>,
        knots_u: KnotVec,
        knots_v: KnotVec,
    ) -> Self {
        let num_knots_u = knots_u.len();
        let num_points_u = unweighted.len();
        let order_u = num_knots_u - num_points_u;
        let degree_u = order_u - 1;

        if degree_u < 1 {
            panic!(
                "Surface would have degree {} in the U-direction. Needs more knots or fewer points.",
                order_u
            )
        }

        let num_knots_v = knots_v.len();
        let num_points_v = unweighted[0].len();
        let order_v = num_knots_v - num_points_v;
        let degree_v = order_v - 1;

        if degree_v < 1 {
            panic!(
                "Surface would have degree {} in the V-direction. Needs more knots or fewer points.",
                order_u
            )
        }

        // Make sure all rows and columns of points have the same length
        for row in unweighted.iter().skip(1) {
            if row.len() != num_points_v {
                panic!("Points must be a rectangular array (all rows must have the same length)");
            }
        }

        knots_u.assert_clamped(degree_u);
        knots_v.assert_clamped(degree_v);

        Self {
            unweighted,
            weighted,
            knots_u,
            knots_v,
            degree_u,
            degree_v,
            order_u,
            order_v,
        }
    }

    pub fn knots_u(&self) -> &KnotVec {
        &self.knots_u
    }

    pub fn knots_v(&self) -> &KnotVec {
        &self.knots_v
    }

    /// Returns the homogeneous point on the surface at the parameter values `u` and `v`.
    pub fn eval_pos(&self, u: f64, v: f64) -> Vec4 {
        // Alg A4.3

        let span_u = self.knots_u.find_span(self.degree_u, u);
        let basis_u = basis(span_u, u, self.degree_u, &self.knots_u);

        let span_v = self.knots_v.find_span(self.degree_v, v);
        let basis_v = basis(span_v, v, self.degree_v, &self.knots_v);

        let mut temp = vec![Vec4::zero(); self.degree_v + 1];
        for l in 0..=self.degree_v {
            for k in 0..=self.degree_u {
                temp[l] += basis_u[k]
                    * self.weighted[span_u - self.degree_u + k][span_v - self.degree_v + l];
            }
        }

        let mut point = Vec4::zero();
        for l in 0..=self.degree_v {
            point += basis_v[l] * temp[l];
        }

        point
    }

    /// Returns the Euclidean point on the surface at the parameter values `u` and `v`, as well as
    /// the normal vector at that point.
    pub fn eval_full(&self, u: f64, v: f64) -> SurfacePoint {
        let ders = self.eval_derivatives(u, v, 1);
        let position = ders[0][0].project();
        let normal = ders[0][1].project().cross(ders[1][0].project()).normalize();

        SurfacePoint { position, normal }
    }

    pub fn eval_derivatives(&self, u: f64, v: f64, num_derivatives: usize) -> Vec<Vec<Vec4>> {
        let weighted_derivatives = surface_derivatives(
            u,
            v,
            &self.weighted,
            self.degree_u,
            self.degree_v,
            &self.knots_u,
            &self.knots_v,
            num_derivatives,
        );

        let mut derivatives = vec![vec![Vec4::zero(); num_derivatives + 1]; num_derivatives + 1];

        for k in 0..=num_derivatives {
            for l in 0..=num_derivatives - k {
                let mut pt3 = weighted_derivatives[k][l].truncate();
                for j in 1..=l {
                    pt3 -=
                        bin(l, j) * weighted_derivatives[0][j].w * derivatives[k][l - j].project();
                }

                for i in 1..=k {
                    pt3 -=
                        bin(k, i) * weighted_derivatives[i][0].w * derivatives[k - i][l].project();
                    let mut v2 = Vec3::zero();
                    for j in 1..=l {
                        v2 += bin(l, j)
                            * weighted_derivatives[i][j].w
                            * derivatives[k - i][l - j].truncate();
                    }
                    pt3 -= bin(k, i) * v2;
                }

                derivatives[k][l] = pt3.to_hpoint(weighted_derivatives[0][0].w);
            }
        }

        derivatives
    }

    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::create_unweighted(
            self.unweighted
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|p| p.transform(transform))
                        .collect::<Vec<_>>()
                })
                .collect(),
            self.knots_u.clone(),
            self.knots_v.clone(),
        )
    }

    pub fn refine_knots(&self, add_knots: Vec<f64>, direction: SurfaceDirection) -> Self {
        match direction {
            SurfaceDirection::U => self.refine_knots_u(add_knots),
            SurfaceDirection::V => self.refine_knots_v(add_knots),
        }
    }

    pub fn refine_knots_u(&self, add_knots: Vec<f64>) -> Self {
        println!("adding {} knots: {:?}", add_knots.len(), add_knots);

        println!(
            "original dimensions = {} x {}",
            self.unweighted.len(),
            self.unweighted[0].len()
        );

        if add_knots.len() == 0 {
            return self.clone();
        }

        let span_a = self.knots_u.find_span(self.degree_u, add_knots[0]);
        let span_b = self
            .knots_u
            .find_span(self.degree_u, add_knots[add_knots.len() - 1])
            + 1;

        println!("span_a = {}", span_a);
        println!("span_b = {}", span_b);

        let m = self.unweighted.len() + self.degree_u;
        let mut out_knots = vec![0.0; m + add_knots.len() + 1];
        let mut out_points = vec![
            vec![Vec4::zero(); self.unweighted[0].len()];
            self.unweighted.len() + add_knots.len()
        ];

        println!(
            "out_points dimensions = {} x {}",
            out_points.len(),
            out_points[0].len()
        );

        println!("m = {}", m);

        for row in 0..=m {
            for k in 0..=span_a - self.degree_u {
                out_points[k][row] = self.weighted[k][row];
            }

            for k in span_b - 1..=self.unweighted.len() {
                println!("k = {}", k);
                println!("k + add_knots.len() = {}", k + add_knots.len());
                println!("row = {}", row);
                out_points[k + add_knots.len()][row] = self.weighted[k][row];
            }
        }

        let mut i = span_b + self.degree_u - 1;
        let mut k = span_b + self.degree_u + add_knots.len() - 1;

        for j in (0..=add_knots.len()).rev() {
            while add_knots[j] <= self.knots_u[i] && i > span_a {
                for row in 0..=m {
                    out_points[k - self.degree_u - 1][row] =
                        self.weighted[i - self.degree_u - 1][row];
                }
                out_knots[k] = self.knots_u[i];
                k = k - 1;
                i = i - 1;
            }

            for row in 0..=m {
                out_points[k - self.degree_u - 1][row] = out_points[k - self.degree_u][row];
            }

            for l in 1..=self.degree_u {
                let ind = k - self.degree_u + l;
                let mut alpha = out_knots[k + l] - add_knots[j];
                if alpha.toleq(0.0) {
                    for row in 0..=m {
                        out_points[ind - 1][row] = out_points[ind][row];
                    }
                } else {
                    alpha = alpha / (out_knots[k + l] - self.knots_u[i - self.degree_u + l]);
                    for row in 0..=m {
                        out_points[ind - 1][row] =
                            alpha * out_points[ind - 1][row] + (1.0 - alpha) * out_points[ind][row];
                    }
                }
            }

            out_knots[k] = add_knots[j];
            k = k - 1;
        }

        Self::create_weighted(out_points, KnotVec::new(out_knots), self.knots_v.clone())
    }

    pub fn refine_knots_v(&self, add_knots: Vec<f64>) -> Self {
        todo!()
    }
}

fn ij_iter(i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..i).flat_map(move |i| (0..j).map(move |j| (i, j)))
}
