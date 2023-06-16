mod builders;

use crate::{basis, knots::KnotVec, HPoint, Pt4};
use cgmath::Matrix4;

pub use builders::*;

#[derive(Debug)]
pub struct Surface {
    unweighted: Vec<Vec<Pt4>>,
    weighted: Vec<Vec<Pt4>>,
    knots_u: KnotVec,
    knots_v: KnotVec,
    order_u: usize,
    order_v: usize,
}
impl Surface {
    pub fn new(unweighted: Vec<Vec<Pt4>>, knots_u: KnotVec, knots_v: KnotVec) -> Self {
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
            order_u,
            order_v,
        }
    }

    pub fn eval(&self, u: f64, v: f64) -> Pt4 {
        ij_iter(self.weighted.len(), self.weighted[0].len())
            .map(|(i, j)| {
                self.weighted[i][j]
                    * basis(&self.knots_u, i, self.order_u, u)
                    * basis(&self.knots_v, j, self.order_v, v)
            })
            .sum()
    }

    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::new(
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
