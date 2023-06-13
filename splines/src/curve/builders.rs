use cgmath::{Matrix4, Rad};
use primitives::Angle;

use crate::{Curve, Pt4};

impl Curve {
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
                Pt4::new(1.0, 0.0, 0.0, 1.0),
                Pt4::new(1.0, half_angle.tan(), 0.0, half_angle.cos()),
                Pt4::new(angle.cos(), angle.sin(), 0.0, 1.0),
            ],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        )
    }
}
