use primitives::{rat, Int, ParamD2, Point4d, Rat};

use crate::{basis_d1, basis_d2, basis_i, basis_s, knot_span, normalize_knots};

#[derive(Debug)]
pub struct Curve {
    points: Vec<Point4d>,
    knots: Vec<Rat>,
    degree: usize,
    order: usize,
}
impl Curve {
    pub fn new(points: Vec<Point4d>, knots: Vec<Rat>) -> Self {
        // Do some validation
        let num_knots = knots.len();
        let num_points = points.len();
        let order = num_knots - num_points;
        let degree = order - 1;
        if degree < 1 {
            panic!(
                "Curve would have degree {} (knots.len() - points.len() - 1). Needs more knots or fewer points.",
                degree
            );
        }

        Self {
            points,
            knots: normalize_knots(knots),
            degree,
            order,
        }
    }

    pub fn eval_s(&self, t: &Rat) -> Point4d {
        self.points
            .iter()
            .enumerate()
            .map(|(j, p)| p * basis_s(&self.knots, j, self.order, t))
            .sum()
    }

    pub fn eval_d1(&self, t: &Rat) -> Point4d {
        (0..self.points.len())
            .map(|j| basis_d1(&self.knots, j, self.order, t) * &self.points[j])
            .sum()
    }

    pub fn eval_d2(&self, t: &ParamD2) -> Point4d {
        (0..self.points.len())
            .map(|j| basis_d2(&self.knots, j, self.order, t) * &self.points[j])
            .sum()
    }

    pub fn eval_i(&self, t: &Rat) -> Point4d {
        let i = knot_span(&self.knots, self.points.len(), t);
        (0..self.points.len())
            .map(|j| basis_i(&self.knots, j, self.order, t, i) * &self.points[j])
            .sum()
    }
}
