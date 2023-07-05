use primitives::{HVec, Vec3};
use splines::{Surface, SurfacePoint};

pub trait SurfaceTessellation {
    /// Calculates `num_sections + 1` points along the surface in both directions.
    /// Sections are equally spaced in the surface's parameter domains.
    fn tessellate_by_params(&self, num_sections: usize) -> Vec<Vec<SurfacePoint>> {
        self.tessellate_by_params_uv(num_sections, num_sections)
    }

    /// Calculates `num_sections_u + 1` points along the surface's U direction
    /// and `num_section_v + 1` points along the surface's V direction.
    /// Sections are equally spaced in the surface's parameter domains.
    fn tessellate_by_params_uv(
        &self,
        num_sections_u: usize,
        num_sections_v: usize,
    ) -> Vec<Vec<SurfacePoint>>;
}

impl SurfaceTessellation for Surface {
    fn tessellate_by_params_uv(
        &self,
        num_sections_u: usize,
        num_sections_v: usize,
    ) -> Vec<Vec<SurfacePoint>> {
        (0..=num_sections_u)
            .map(|i| {
                let u = i as f64 / num_sections_u as f64;
                (0..=num_sections_v)
                    .map(|j| self.eval_full(u, j as f64 / num_sections_v as f64))
                    .collect()
            })
            .collect()
    }
}
