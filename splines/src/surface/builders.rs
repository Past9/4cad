use crate::{Curve, Mat4, SplineHelpers4, Surface, Vec3};

impl Surface {
    pub fn rule_curve(curve: Curve, direction: Vec3) -> Self {
        let transform = Mat4::from_translation(direction);

        let Curve {
            points: row1,
            knots: knots_v,
            ..
        } = curve;

        let mut row2 = row1.clone();
        row2.iter_mut().for_each(|p| p.transform(transform));

        Self::new(vec![row1, row2], vec![0.0, 0.0, 1.0, 1.0], knots_v)
    }
}
