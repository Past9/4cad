use cgmath::{Matrix4, Point3, Rad, Rotation3, Transform, Transform3, Vector3};
use primitives::{Angle, Point4d};

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

    pub fn transform(&mut self, transform: Matrix4<f64>) {
        for point in self.points.iter_mut() {
            let xyz = Point3::new(point.x, point.y, point.z);
            let xyz = transform.transform_point(xyz);
            point.x = xyz.x;
            point.y = xyz.y;
            point.z = xyz.z;
        }
    }

    pub fn arc(angle: Angle) -> Curve {
        let mut full_points = vec![];
        let mut full_knots = vec![0.0, 0.0, 0.0];

        let num_sections = (angle / Angle::deg(120.0)).ceil().abs();
        let section_angle = angle / num_sections;

        for s in 0..num_sections as usize {
            let start_angle = section_angle * s as f64;
            let mut section = Self::arc_section(section_angle);
            section.transform(Matrix4::from_angle_z(Rad(start_angle.0)));

            if s == 0 {
                full_points.extend(section.points);
            } else {
                let knot = (s) as f64 / num_sections as f64;
                full_knots.extend([knot, knot]);
                full_points.extend(section.points.into_iter().skip(1));
            }
        }

        full_knots.extend([1.0, 1.0, 1.0]);

        Self::new(full_points, full_knots)
    }

    fn arc_section(angle: Angle) -> Curve {
        let half_angle = angle / 2.0;

        Curve::new(
            vec![
                Point4d::new(1.0, 1.0, 0.0, 0.0),
                Point4d::new(half_angle.cos(), 1.0, half_angle.tan(), 0.0),
                Point4d::new(1.0, angle.cos(), angle.sin(), 0.0),
            ],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        )
    }
}
