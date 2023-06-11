use primitives::Point4d;

use crate::{basis, normalize_knots};

#[derive(Debug)]
pub struct Curve {
    points: Vec<Point4d>,
    knots: Vec<f64>,
    order: usize,
}
impl Curve {
    pub fn new(points: Vec<Point4d>, knots: Vec<f64>) -> Self {
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
            order,
        }
    }

    pub fn eval(&self, t: f64) -> Point4d {
        self.points
            .iter()
            .enumerate()
            .map(|(j, p)| {
                let basis = basis(&self.knots, j, self.order, t);
                Point4d {
                    w: p.w * basis,
                    x: p.w * p.x * basis,
                    y: p.w * p.y * basis,
                    z: p.w * p.z * basis,
                }
            })
            .sum()
    }
}
