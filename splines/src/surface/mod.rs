mod builders;

use crate::{basis, knots::KnotVec, HPoint, Pt4};
use cgmath::{Matrix4, Zero};

pub use builders::*;

#[derive(Debug)]
pub struct Surface {
    unweighted: Vec<Vec<Pt4>>,
    weighted: Vec<Vec<Pt4>>,
    knots_u: KnotVec,
    knots_v: KnotVec,
    degree_u: usize,
    degree_v: usize,
    order_u: usize,
    order_v: usize,
}
impl Surface {
    pub fn unweighted(unweighted: Vec<Vec<Pt4>>, knots_u: KnotVec, knots_v: KnotVec) -> Self {
        let weighted: Vec<Vec<Pt4>> = unweighted
            .iter()
            .map(|row| row.iter().map(HPoint::weight).collect())
            .collect();
        Self::create(unweighted, weighted, knots_u, knots_v)
    }

    pub fn weighted(weighted: Vec<Vec<Pt4>>, knots_u: KnotVec, knots_v: KnotVec) -> Self {
        let unweighted: Vec<Vec<Pt4>> = weighted
            .iter()
            .map(|row| row.iter().map(HPoint::unweight).collect())
            .collect();
        Self::create(unweighted, weighted, knots_u, knots_v)
    }

    pub fn create(
        unweighted: Vec<Vec<Pt4>>,
        weighted: Vec<Vec<Pt4>>,
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

    pub fn eval(&self, u: f64, v: f64) -> Pt4 {
        // Alg A4.3

        let span_u = self.knots_u.find_span(self.degree_u, u);
        let basis_u = basis(span_u, u, self.degree_u, &self.knots_u);

        let span_v = self.knots_v.find_span(self.degree_v, v);
        let basis_v = basis(span_v, v, self.degree_v, &self.knots_v);

        let mut temp = vec![Pt4::zero(); self.degree_v + 1];
        for l in 0..=self.degree_v {
            for k in 0..=self.degree_u {
                temp[l] += basis_u[k]
                    * self.weighted[span_u - self.degree_u + k][span_v - self.degree_v + l];
            }
        }

        let mut point = Pt4::zero();
        for l in 0..=self.degree_v {
            point += basis_v[l] * temp[l];
        }

        point
    }

    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::unweighted(
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
}

fn ij_iter(i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..i).flat_map(move |i| (0..j).map(move |j| (i, j)))
}
