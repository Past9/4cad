mod builders;
use crate::{basis, bin, knots::KnotVec, refine_knots, surface_derivatives, transpose, Vec4};
pub use builders::*;
use cgmath::{InnerSpace, Matrix4, Zero};
use primitives::{EVec, HVec, Vec3};

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

    pub fn num_pts_u(&self) -> usize {
        self.unweighted.len()
    }

    pub fn num_pts_v(&self) -> usize {
        self.unweighted[0].len()
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
        if add_knots.len() == 0 {
            return self.clone();
        }

        let weighted = transpose(self.weighted.clone());

        let mut out_points = vec![];
        let mut out_knots: KnotVec = KnotVec::empty();

        for row in weighted.into_iter() {
            let (refined_points, refined_knots) =
                refine_knots(self.degree_u, &self.knots_u, &row, &add_knots);

            out_points.push(refined_points);
            out_knots = refined_knots;
        }

        Self::create_weighted(transpose(out_points), out_knots, self.knots_v.clone())
    }

    pub fn refine_knots_v(&self, add_knots: Vec<f64>) -> Self {
        if add_knots.len() == 0 {
            return self.clone();
        }

        let mut out_points = vec![];
        let mut out_knots: KnotVec = KnotVec::empty();

        for row in self.weighted.clone().into_iter() {
            let (refined_points, refined_knots) =
                refine_knots(self.degree_v, &self.knots_v, &row, &add_knots);

            out_points.push(refined_points);
            out_knots = refined_knots;
        }

        Self::create_weighted(out_points, self.knots_u.clone(), out_knots)
    }
}

fn ij_iter(i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..i).flat_map(move |i| (0..j).map(move |j| (i, j)))
}
