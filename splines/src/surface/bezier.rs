use crate::Surface;
use cgmath::Zero;
use once_cell::unsync::OnceCell;
use primitives::{HVec, Vec3};

use super::bsp::BspTree;

#[derive(Debug, Clone)]
struct CornerDerivatives {
    umin_vmin: (Vec3, Vec3),
    umax_vmin: (Vec3, Vec3),
    umax_vmax: (Vec3, Vec3),
    umin_vmax: (Vec3, Vec3),
}

#[derive(Debug, Clone)]
pub struct SurfaceBezierComponent {
    pub param_span_u: (f64, f64),
    pub param_span_v: (f64, f64),
    pub surface: Box<Surface>,
    corner_derivatives: OnceCell<CornerDerivatives>,
}
impl SurfaceBezierComponent {
    pub fn new(surface: Surface, param_span_u: (f64, f64), param_span_v: (f64, f64)) -> Self {
        Self {
            param_span_u,
            param_span_v,
            surface: Box::new(surface),
            corner_derivatives: OnceCell::new(),
        }
    }

    pub(crate) fn split_u(&self) -> (Self, Self) {
        let refined = self
            .surface
            .refine_knots_u((0..=self.surface.degree_u).map(|_| 0.5).collect());

        let middle_knot = (self.param_span_u.0 + self.param_span_u.1) / 2.0;

        let n = Self::new(
            Surface::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .take(refined.unweighted.len() / 2)
                    .cloned()
                    .collect(),
            ),
            (self.param_span_u.0, middle_knot),
            self.param_span_v,
        );

        let s = Self::new(
            Surface::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .skip(refined.unweighted.len() / 2)
                    .cloned()
                    .collect(),
            ),
            (middle_knot, self.param_span_u.0),
            self.param_span_v,
        );

        (n, s)
    }

    pub(crate) fn split_v(&self) -> (Self, Self) {
        let refined = self
            .surface
            .refine_knots_v((0..=self.surface.degree_u).map(|_| 0.5).collect());

        let middle_knot = (self.param_span_v.0 + self.param_span_v.1) / 2.0;

        let w = Self::new(
            Surface::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .map(|row| row.iter().take(row.len() / 2).cloned().collect::<Vec<_>>())
                    .collect(),
            ),
            self.param_span_u,
            (self.param_span_v.0, middle_knot),
        );

        let e = Self::new(
            Surface::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .map(|row| row.iter().skip(row.len() / 2).cloned().collect::<Vec<_>>())
                    .collect(),
            ),
            self.param_span_u,
            (middle_knot, self.param_span_v.0),
        );

        (w, e)
    }

    fn corner_derivatives(&self) -> &CornerDerivatives {
        self.corner_derivatives.get_or_init(|| {
            let umin_vmin = self.surface.eval_derivatives(0.0, 0.0, 1)[1].clone();
            let umax_vmin = self.surface.eval_derivatives(1.0, 0.0, 1)[1].clone();
            let umax_vmax = self.surface.eval_derivatives(1.0, 1.0, 1)[1].clone();
            let umin_vmax = self.surface.eval_derivatives(0.0, 1.0, 1)[1].clone();

            CornerDerivatives {
                umin_vmin: (umin_vmin[0].project(), umin_vmin[1].project()),
                umax_vmin: (umax_vmin[0].project(), umax_vmin[1].project()),
                umax_vmax: (umax_vmax[0].project(), umax_vmax[1].project()),
                umin_vmax: (umin_vmax[0].project(), umin_vmax[1].project()),
            }
        })
    }
}

impl BspTree<SurfaceBezierComponent> {
    pub fn split_until_condition(
        &self,
        split_u: fn(patch: &SurfaceBezierComponent) -> bool,
        split_v: fn(patch: &SurfaceBezierComponent) -> bool,
    ) -> Self {
        match self {
            BspTree::EW { e, w } => BspTree::EW {
                e: Box::new(e.split_until_condition(split_u, split_v)),
                w: Box::new(w.split_until_condition(split_u, split_v)),
            },
            BspTree::NS { n, s } => BspTree::NS {
                n: Box::new(n.split_until_condition(split_u, split_v)),
                s: Box::new(s.split_until_condition(split_u, split_v)),
            },
            BspTree::Cell(patch) => {
                if split_u(patch) {
                    let (n, s) = patch.split_u();
                    BspTree::NS {
                        n: Box::new(BspTree::Cell(n)),
                        s: Box::new(BspTree::Cell(s)),
                    }
                    .split_until_condition(split_u, split_v)
                } else if split_v(patch) {
                    let (w, e) = patch.split_v();
                    BspTree::EW {
                        w: Box::new(BspTree::Cell(w)),
                        e: Box::new(BspTree::Cell(e)),
                    }
                    .split_until_condition(split_u, split_v)
                } else {
                    BspTree::Cell(patch.clone())
                }
            }
        }
    }
}
