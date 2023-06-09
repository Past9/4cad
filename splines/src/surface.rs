use crate::{basis_d1, basis_d2, basis_i, basis_s, knot_span, normalize_knots};
use primitives::{ParamD2, Point4d, Rat};

#[derive(Debug)]
pub struct Surface {
    points: Vec<Vec<Point4d>>,
    knots_u: Vec<Rat>,
    knots_v: Vec<Rat>,
    degree_u: usize,
    degree_v: usize,
    order_u: usize,
    order_v: usize,
}
impl Surface {
    pub fn new(points: Vec<Vec<Point4d>>, knots_u: Vec<Rat>, knots_v: Vec<Rat>) -> Self {
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
            degree_u,
            degree_v,
            order_u,
            order_v,
        }
    }

    pub fn eval_s(&self, u: &Rat, v: &Rat) -> Point4d {
        ij_iter(self.points.len(), self.points[0].len())
            .map(|(i, j)| {
                basis_s(&self.knots_u, i, self.order_u, u)
                    * basis_s(&self.knots_v, j, self.order_v, v)
                    * &self.points[i][j]
            })
            .sum()
    }

    pub fn eval_d1(&self, u: &Rat, v: &Rat) -> Point4d {
        ij_iter(self.points.len(), self.points[0].len())
            .map(|(i, j)| {
                basis_d1(&self.knots_u, i, self.order_u, u)
                    * basis_d1(&self.knots_v, j, self.order_v, v)
                    * &self.points[i][j]
            })
            .sum()
    }

    pub fn eval_d2(&self, u: &ParamD2, v: &ParamD2) -> Point4d {
        ij_iter(self.points.len(), self.points[0].len())
            .map(|(i, j)| {
                basis_d2(&self.knots_u, i, self.order_u, u)
                    * basis_d2(&self.knots_v, j, self.order_v, v)
                    * &self.points[i][j]
            })
            .sum()
    }

    pub fn eval_i(&self, u: &Rat, v: &Rat) -> Point4d {
        let knot_span_u = knot_span(&self.knots_u, self.points.len(), u);
        let knot_span_v = knot_span(&self.knots_v, self.points[0].len(), v);
        ij_iter(self.points.len(), self.points[0].len())
            .map(|(i, j)| {
                basis_i(&self.knots_u, i, self.order_u, u, knot_span_u)
                    * basis_i(&self.knots_v, j, self.order_v, v, knot_span_v)
                    * &self.points[i][j]
            })
            .sum()
    }
}

fn ij_iter(i: usize, j: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..i).flat_map(move |i| (0..j).map(move |j| (i, j)))
}
