use crate::{basis, normalize_knots};
use primitives::Point4d;

#[derive(Debug)]
pub struct Surface {
    points: Vec<Vec<Point4d>>,
    knots_u: Vec<f64>,
    knots_v: Vec<f64>,
    order_u: usize,
    order_v: usize,
}
impl Surface {
    pub fn new(points: Vec<Vec<Point4d>>, knots_u: Vec<f64>, knots_v: Vec<f64>) -> Self {
        let num_knots_u = knots_u.len();
        let num_points_u = points.len();
        let order_u = num_knots_u - num_points_u;
        let degree_u = order_u - 1;

        if degree_u < 1 {
            panic!(
                "Surface would have degree {} in the U-direction. Needs more knots or fewer points.",
                order_u
            )
        }

        let num_knots_v = knots_v.len();
        let num_points_v = points[0].len();
        let order_v = num_knots_v - num_points_v;
        let degree_v = order_v - 1;

        if degree_v < 1 {
            panic!(
                "Surface would have degree {} in the V-direction. Needs more knots or fewer points.",
                order_u
            )
        }

        // Make sure all rows and columns of points have the same length
        for row in points.iter().skip(1) {
            if row.len() != num_points_v {
                panic!("Points must be a rectangular array (all rows must have the same length)");
            }
        }

        Self {
            points,
            knots_u: normalize_knots(knots_u),
            knots_v: normalize_knots(knots_v),
            order_u,
            order_v,
        }
    }

    pub fn eval(&self, u: f64, v: f64) -> Point4d {
        ij_iter(self.points.len(), self.points[0].len())
            .map(|(i, j)| {
                basis(&self.knots_u, i, self.order_u, u)
                    * basis(&self.knots_v, j, self.order_v, v)
                    * &self.points[i][j]
            })
            .sum()
    }
}

fn ij_iter(i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..i).flat_map(move |i| (0..j).map(move |j| (i, j)))
}
