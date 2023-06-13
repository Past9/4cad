mod builders;

use cgmath::{Matrix4, Point3, Rad, Transform};
use primitives::{Angle, Point4d};

use crate::{basis, normalize_knots};
pub use builders::*;

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

        let knots = normalize_knots(knots);

        for i in 0..order {
            if knots[i] != 0.0 {
                panic!(
                    "Unclamped curve: First {} normalized knots must be 0, but knots are {:?}",
                    order, knots
                );
            }
        }

        for i in 0..order {
            if knots[knots.len() - i - 1] != 1.0 {
                panic!(
                    "Unclamped curve: Last {} normalized knots must be 1, but knots are {:?}",
                    order, knots
                );
            }
        }

        Self {
            points,
            knots,
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

    pub fn transform(&mut self, transform: Matrix4<f64>) {
        for point in self.points.iter_mut() {
            let xyz = Point3::new(point.x, point.y, point.z);
            let xyz = transform.transform_point(xyz);
            point.x = xyz.x;
            point.y = xyz.y;
            point.z = xyz.z;
        }
    }
}
