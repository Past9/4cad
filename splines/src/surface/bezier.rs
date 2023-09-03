use crate::Surface;
use cgmath::Zero;
use once_cell::unsync::OnceCell;
use primitives::{HVec, Vec3};

struct CornerDerivatives {
    umin_vmin: (Vec3, Vec3),
    umax_vmin: (Vec3, Vec3),
    umax_vmax: (Vec3, Vec3),
    umin_vmax: (Vec3, Vec3),
}

pub struct SurfaceBezierComponent {
    pub param_span_u: (f64, f64),
    pub param_span_v: (f64, f64),
    pub surface: Surface,
    corner_derivatives: OnceCell<CornerDerivatives>,
}
impl SurfaceBezierComponent {
    pub fn new(surface: Surface, param_span_u: (f64, f64), param_span_v: (f64, f64)) -> Self {
        Self {
            param_span_u,
            param_span_v,
            surface,
            corner_derivatives: OnceCell::new(),
        }
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
