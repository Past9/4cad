use crate::Surface;
use cgmath::Zero;
use once_cell::unsync::OnceCell;
use primitives::{HVec, Vec3};

use super::bsp::BspTree;

#[derive(Debug, Clone)]
pub struct SurfaceBezierComponent {
    pub param_span_u: (f64, f64),
    pub param_span_v: (f64, f64),
    pub surface: Box<Surface>,
}
impl SurfaceBezierComponent {
    pub fn new(surface: Surface, param_span_u: (f64, f64), param_span_v: (f64, f64)) -> Self {
        Self {
            param_span_u,
            param_span_v,
            surface: Box::new(surface),
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
            (middle_knot, self.param_span_u.1),
            self.param_span_v,
        );

        (n, s)
    }

    pub(crate) fn split_v(&self) -> (Self, Self) {
        let refined = self
            .surface
            .refine_knots_v((0..=self.surface.degree_v).map(|_| 0.5).collect());

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
            (middle_knot, self.param_span_v.1),
        );

        (w, e)
    }
}

impl BspTree<SurfaceBezierComponent> {
    pub fn split_while_condition(
        &self,
        split_u: fn(patch: &SurfaceBezierComponent) -> bool,
        split_v: fn(patch: &SurfaceBezierComponent) -> bool,
    ) -> Self {
        match self {
            BspTree::EW { e, w } => BspTree::EW {
                e: Box::new(e.split_while_condition(split_u, split_v)),
                w: Box::new(w.split_while_condition(split_u, split_v)),
            },
            BspTree::NS { n, s } => BspTree::NS {
                n: Box::new(n.split_while_condition(split_u, split_v)),
                s: Box::new(s.split_while_condition(split_u, split_v)),
            },
            BspTree::Cell(patch) => {
                if split_u(patch) {
                    let (n, s) = patch.split_u();
                    BspTree::NS {
                        n: Box::new(BspTree::Cell(n).split_while_condition(split_u, split_v)),
                        s: Box::new(BspTree::Cell(s).split_while_condition(split_u, split_v)),
                    }
                    .split_while_condition(split_u, split_v)
                } else if split_v(patch) {
                    let (w, e) = patch.split_v();
                    BspTree::EW {
                        w: Box::new(BspTree::Cell(w).split_while_condition(split_u, split_v)),
                        e: Box::new(BspTree::Cell(e).split_while_condition(split_u, split_v)),
                    }
                    .split_while_condition(split_u, split_v)
                } else {
                    BspTree::Cell(patch.clone())
                }
            }
        }
    }
}
