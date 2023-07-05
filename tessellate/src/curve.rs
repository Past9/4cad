use primitives::{HVec, Vec3};
use splines::Curve;

pub trait CurveTessellation {
    /// Calculates `num_sections + 1` points along the curve that are equally spaced
    /// in the curve's parameter domain.
    fn tessellate_by_param(&self, num_sections: usize) -> Vec<Vec3>;
}

impl CurveTessellation for Curve {
    fn tessellate_by_param(&self, num_sections: usize) -> Vec<Vec3> {
        (0..=num_sections)
            .map(|i| self.eval_pos(i as f64 / num_sections as f64).project())
            .collect()
    }
}
