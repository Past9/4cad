use crate::{line_to_point_perpendicular, Curve};
use cgmath::InnerSpace;
use once_cell::unsync::OnceCell;
use primitives::{HVec, TolEq, Vec3};

const STRAIGHT_BEZIER_THRESHOLD: f64 = 0.99;
const BEZIER_SPLIT_RECURSION_LIMIT: usize = 12;

#[derive(Debug, Clone)]
pub struct CurveBezierComponent {
    pub param_span: (f64, f64),
    pub curve: Curve,
    end_derivatives: OnceCell<(Vec3, Vec3)>,
}
impl CurveBezierComponent {
    pub fn new(curve: Curve, param_span: (f64, f64)) -> Self {
        Self {
            param_span,
            curve,
            end_derivatives: OnceCell::new(),
        }
    }

    fn end_derivatives(&self) -> &(Vec3, Vec3) {
        self.end_derivatives.get_or_init(|| {
            let start_der = self.curve.eval_derivatives(0.0, 1)[1].project();
            let end_der = self.curve.eval_derivatives(1.0, 1)[1].project();
            (start_der, end_der)
        })
    }

    pub fn estimate_projection_parameter(&self, point: Vec3) -> Option<f64> {
        let start = self.curve.weighted[0].project();
        let end = self.curve.weighted[self.curve.weighted.len() - 1].project();
        let line = end - start;

        let line_to_point = line_to_point_perpendicular(start, end, point);
        let point_on_line = point + line_to_point;
        let fraction_of_line = (point_on_line - start).dot(line.normalize()) / line.magnitude();
        let param = self.param_span.0 + (self.param_span.1 - self.param_span.0) * fraction_of_line;

        return Some(param);
    }

    pub fn has_perpendicular_projection(&self, point: Vec3) -> bool {
        let p0 = self.curve.weighted[0].project();
        let p1 = self.curve.weighted[1].project();

        if p0.toleq(point) || p1.toleq(point) {
            return true;
        }

        let pn = self.curve.weighted[self.curve.weighted.len() - 1].project();
        let pnsub1 = self.curve.weighted[self.curve.weighted.len() - 2].project();

        let p0p = (point - p0).normalize();
        let p0p1 = (p1 - p0).normalize();
        let ppn = (pn - point).normalize();
        let pnsub1pn = (pn - pnsub1).normalize();
        let pnp0 = (p0 - pn).normalize();
        let pnp = (point - pn).normalize();

        let r1 = p0p.dot(p0p1);
        let r2 = ppn.dot(pnsub1pn);
        let r3 = pnp0.dot(pnp);
        let r4 = pnp0.dot(p0p);

        (r1 >= 0.0 && r2 >= 0.0) || (r3 * r4 <= 0.0)
    }

    fn straightness(&self) -> f64 {
        let (start_der, end_der) = self.end_derivatives();
        start_der.normalize().dot(end_der.normalize())
    }

    fn is_straight(&self) -> bool {
        self.straightness() >= STRAIGHT_BEZIER_THRESHOLD
    }

    pub fn split_until_convex(&self) -> Vec<CurveBezierComponent> {
        self.do_split_until_convex(BEZIER_SPLIT_RECURSION_LIMIT)
    }

    pub fn split_until_straight(&self) -> Vec<CurveBezierComponent> {
        self.do_split_until_straight(BEZIER_SPLIT_RECURSION_LIMIT)
    }

    fn do_split_until_convex(&self, rec_limit: usize) -> Vec<CurveBezierComponent> {
        if self.curve.is_convex() || rec_limit == 0 {
            vec![self.clone()]
        } else {
            let (bez1, bez2) = self.split();

            bez1.do_split_until_convex(rec_limit - 1)
                .into_iter()
                .chain(bez2.do_split_until_convex(rec_limit - 1).into_iter())
                .collect()
        }
    }

    fn do_split_until_straight(&self, rec_limit: usize) -> Vec<CurveBezierComponent> {
        if self.is_straight() || rec_limit == 0 {
            vec![self.clone()]
        } else {
            let (bez1, bez2) = self.split();

            bez1.do_split_until_straight(rec_limit - 1)
                .into_iter()
                .chain(bez2.do_split_until_straight(rec_limit - 1).into_iter())
                .collect()
        }
    }

    fn split(&self) -> (CurveBezierComponent, CurveBezierComponent) {
        let refined = self
            .curve
            .refine_knots((0..=self.curve.degree).map(|_| 0.5).collect());

        let middle_knot = (self.param_span.0 + self.param_span.1) / 2.0;

        let bez1 = Self::new(
            Curve::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .take(refined.unweighted.len() / 2)
                    .cloned()
                    .collect(),
            ),
            (self.param_span.0, middle_knot),
        );

        let bez2 = Self::new(
            Curve::create_unweighted_bezier(
                refined
                    .unweighted
                    .iter()
                    .skip(refined.unweighted.len() / 2)
                    .cloned()
                    .collect(),
            ),
            (middle_knot, self.param_span.1),
        );

        (bez1, bez2)
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{vec3, vec4};
    use primitives::HVec;

    use crate::{Curve, CurveBezierComponent};

    #[test]
    fn identifies_points_with_perpendicular_projections() {
        let bezier = CurveBezierComponent::new(
            Curve::create_unweighted_bezier(vec![
                vec4(-1.0, 0.0, 0.0, 1.0),
                vec4(0.0, -1.0, 0.0, 1.0),
                vec4(1.0, 0.0, 0.0, 1.0),
            ]),
            (0.0, 1.0),
        );

        // Below
        assert!(bezier.has_perpendicular_projection(vec3(0.0, 0.0, 0.0)));

        // Above
        assert!(bezier.has_perpendicular_projection(vec3(0.0, 1.0, 0.0)));

        // Right
        assert!(!bezier.has_perpendicular_projection(vec3(2.0, 0.0, 0.0)));

        // Left
        assert!(!bezier.has_perpendicular_projection(vec3(-2.0, 0.0, 0.0)));

        // On curve
        assert!(bezier.has_perpendicular_projection(bezier.curve.eval_pos(0.0).project()));
        assert!(bezier.has_perpendicular_projection(bezier.curve.eval_pos(0.25).project()));
        assert!(bezier.has_perpendicular_projection(bezier.curve.eval_pos(0.5).project()));
        assert!(bezier.has_perpendicular_projection(bezier.curve.eval_pos(0.75).project()));
        assert!(bezier.has_perpendicular_projection(bezier.curve.eval_pos(0.5).project()));
    }
}
