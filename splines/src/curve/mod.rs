mod builders;

use crate::{
    basis, bin, curve_derivatives, knots::KnotVec, line_to_point_perpendicular, nurbs_to_beziers,
    Vec4,
};
use cgmath::{InnerSpace, Matrix4, Zero};
use once_cell::unsync::OnceCell;
use primitives::{EVec, HVec, TolEq, Vec3, TOL};
use std::cmp::{max, min};

pub use builders::*;

const MAX_NEWTON_ITER: usize = 1000;
const ZERO_COS_TOL: f64 = TOL / 100.0;
const STRAIGHT_BEZIER_THRESHOLD: f64 = 0.9999;
const BEZIER_SPLIT_RECURSION_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy)]
pub enum PolygonKind {
    /// Polygon has no crossing edges and is convex
    SimpleConvex,

    /// Polygon has no crossing edges and is concave
    SimpleConcave,

    /// Polygon has crossing edges
    Complex,
}

enum ProjectionKind {
    Projection,
    Inversion,
}

#[derive(Debug, Clone)]
pub struct BezierComponent {
    pub param_span: (f64, f64),
    pub curve: Curve,
    end_derivatives: OnceCell<(Vec3, Vec3)>,
}
impl BezierComponent {
    pub fn new(curve: Curve, param_start: f64, param_end: f64) -> Self {
        Self {
            param_span: (param_start, param_end),
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

        let line_to_point = line_to_point_perpendicular(start, end, point);
        let point_on_line = point - line_to_point;
        let fraction_of_line = (point_on_line - start).dot(end - start);
        let param = self.param_span.0 + (self.param_span.1 - self.param_span.0) * fraction_of_line;

        if param >= self.param_span.0 && param <= self.param_span.1 {
            Some(param)
        } else {
            None
        }
    }

    pub fn has_perpendicular_projection(&self, point: Vec3) -> bool {
        let p0 = self.curve.weighted[0].project();
        let p1 = self.curve.weighted[1].project();
        let pn = self.curve.weighted[self.curve.weighted.len() - 1].project();
        let pnsub1 = self.curve.weighted[self.curve.weighted.len() - 2].project();

        let p0p = point - p0;
        let p0p1 = p1 - p0;
        let ppn = pn - point;
        let pnsub1pn = pn - pnsub1;
        let pnp0 = p0 - pn;
        let pnp = point - pn;

        let r1 = p0p.dot(p0p1);
        let r2 = ppn.dot(pnsub1pn);
        let r3 = pnp0.dot(pnp);
        let r4 = pnp0.dot(p0p);

        (r1 > 0.0 && r2 > 0.0) || (r3 * r4 < 0.0)
    }

    fn straightness(&self) -> f64 {
        let (start_der, end_der) = self.end_derivatives();
        start_der.normalize().dot(end_der.normalize())
    }

    fn is_straight(&self) -> bool {
        self.straightness() >= STRAIGHT_BEZIER_THRESHOLD
    }

    fn split_until_convex(&self) -> Vec<BezierComponent> {
        self.do_split_until_convex(BEZIER_SPLIT_RECURSION_LIMIT)
    }

    pub fn split_until_straight(&self) -> Vec<BezierComponent> {
        self.do_split_until_straight(BEZIER_SPLIT_RECURSION_LIMIT)
    }

    fn do_split_until_convex(&self, rec_limit: usize) -> Vec<BezierComponent> {
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

    fn do_split_until_straight(&self, rec_limit: usize) -> Vec<BezierComponent> {
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

    fn split(&self) -> (BezierComponent, BezierComponent) {
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
            self.param_span.0,
            middle_knot,
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
            middle_knot,
            self.param_span.1,
        );

        (bez1, bez2)
    }
}

#[derive(Debug)]
pub struct CurveProjectionResult {
    pub u: f64,
    pub pos: Vec4,
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct Curve {
    weighted: Vec<Vec4>,
    unweighted: Vec<Vec4>,
    knots: KnotVec,
    order: usize,
    degree: usize,
    is_convex: OnceCell<bool>,
    beziers: OnceCell<Vec<BezierComponent>>,
    convex_beziers: OnceCell<Vec<BezierComponent>>,
    straight_beziers: OnceCell<Vec<BezierComponent>>,
}
impl Curve {
    pub fn create_unweighted(unweighted: Vec<Vec4>, knots: KnotVec) -> Self {
        let weighted = unweighted.iter().map(HVec::weight).collect();
        Self::create(unweighted, weighted, knots)
    }

    pub fn create_weighted(weighted: Vec<Vec4>, knots: KnotVec) -> Self {
        let unweighted = weighted.iter().map(HVec::unweight).collect();
        Self::create(unweighted, weighted, knots)
    }

    pub fn create_unweighted_bezier(unweighted: Vec<Vec4>) -> Self {
        let knots = KnotVec::bezier(unweighted.len() - 1);
        let weighted = unweighted.iter().map(HVec::weight).collect();
        Self::create(unweighted, weighted, knots)
    }

    pub fn create_weighted_bezier(weighted: Vec<Vec4>) -> Self {
        let knots = KnotVec::bezier(weighted.len() - 1);
        let unweighted = weighted.iter().map(HVec::unweight).collect();
        Self::create(unweighted, weighted, knots)
    }

    fn create(unweighted: Vec<Vec4>, weighted: Vec<Vec4>, knots: KnotVec) -> Self {
        let num_knots = knots.len();
        let num_points = unweighted.len();
        let order = num_knots - num_points;
        let degree = order - 1;
        if degree < 1 {
            panic!(
                "Curve would have degree {} (knots.len() - points.len() - 1). Needs more knots or fewer points.",
                degree
            );
        }

        knots.assert_clamped(degree);

        Self {
            weighted,
            unweighted,
            knots,
            order,
            degree,
            is_convex: OnceCell::new(),
            beziers: OnceCell::new(),
            convex_beziers: OnceCell::new(),
            straight_beziers: OnceCell::new(),
        }
    }

    /// Returns the piecewise bezier decomposition of this curve.
    pub fn beziers(&self) -> &[BezierComponent] {
        self.beziers
            .get_or_init(|| nurbs_to_beziers(&self.weighted, self.degree, &self.knots))
    }

    /// Returns the piecewise bezier decomposition of this curve, subdivided
    /// until each bezier curve has a convex control polygon.
    pub fn convex_beziers(&self) -> &[BezierComponent] {
        self.convex_beziers.get_or_init(|| {
            let mut convex_beziers = vec![];

            for bezier in self.beziers().iter() {
                convex_beziers.extend(bezier.split_until_convex());
            }

            convex_beziers
        })
    }

    pub fn straight_beziers(&self) -> &[BezierComponent] {
        self.straight_beziers.get_or_init(|| {
            let mut straight_beziers = vec![];

            for convex_bezier in self.convex_beziers().iter() {
                straight_beziers.extend(convex_bezier.split_until_straight());
            }

            straight_beziers
        })
    }

    /// Returns whether the curve has a convex control polygon
    pub fn is_convex(&self) -> bool {
        // Implements is_valid_polygon (algorithm 1) from "Point inversion
        // and projection for NURBS curve: Control polygon approach"

        *self.is_convex.get_or_init(|| {
            let poly = self
                .unweighted
                .iter()
                .map(|pt| pt.project())
                .collect::<Vec<_>>();

            let n = poly.len() - 1;
            for i in 1..n {
                let pt_prev = poly[i - 1];
                let pt = poly[i];
                let pt_next = poly[i + 1];

                // Compute the projection vector V1Pi
                let v1pi = line_to_point_perpendicular(pt_prev, pt_next, pt);

                let r = if i < n / 2 {
                    v1pi.dot(line_to_point_perpendicular(pt_prev, pt_next, poly[n]))
                } else {
                    v1pi.dot(line_to_point_perpendicular(pt_prev, pt_next, poly[0]))
                };

                if r > 0.0 {
                    return false;
                }
            }

            true
        })
    }

    pub fn ref_weighted(&self) -> &[Vec4] {
        &self.weighted
    }

    pub fn ref_unweighted(&self) -> &[Vec4] {
        &self.unweighted
    }

    pub fn take_weighted(self) -> Vec<Vec4> {
        self.weighted
    }

    pub fn take_unweighted(self) -> Vec<Vec4> {
        self.unweighted
    }

    pub fn num_pts(&self) -> usize {
        self.unweighted.len()
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn knots(&self) -> &KnotVec {
        &self.knots
    }

    /// Returns the point on the curve at the parameter value `u`.
    pub fn eval_pos(&self, u: f64) -> Vec4 {
        // Alg A4.1
        let span = self.knots.find_span(self.degree, u);
        let basis = basis(span, u, self.degree, &self.knots);
        let mut point = Vec4::zero();
        for j in 0..=self.degree {
            point += basis[j] * self.weighted[span - self.degree + j];
        }

        point
    }

    /// Evaluates the position of the curve as the first `num_derivatives` derivatives at `u`.
    /// Returns a `Vec<Vec4>` where the first element (index 0) is the point on the curve at `u`
    /// (the "zero-th derivative"), the second element (index 1) is the vector of the first
    /// derivative, the third (index 2) is the vector of the second derivative, and so on.
    pub fn eval_derivatives(&self, u: f64, num_derivatives: usize) -> Vec<Vec4> {
        let weighted_derivatives =
            curve_derivatives(u, &self.weighted, self.degree, &self.knots, num_derivatives);

        let mut derivatives = vec![Vec4::zero(); num_derivatives + 1];

        for k in 0..=num_derivatives {
            let mut pt3 = weighted_derivatives[k].truncate();
            for i in 1..=k {
                pt3 -= derivatives[k - i].project() * bin(k, i) * weighted_derivatives[i].w;
            }
            derivatives[k] = pt3.to_hpoint(weighted_derivatives[0].w);
        }

        derivatives
    }

    /// Applies a matrix transformation to the curve.
    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::create_unweighted(
            self.unweighted
                .iter()
                .map(|p| p.transform(transform))
                .collect(),
            self.knots.clone(),
        )
    }

    /// Adds the necessary knots so that the curve's knot vector
    /// matches `final_knots`. Does not remove knots, so if there
    /// are any knots in the current knot vector that are not in
    /// `final_knots`, and error will be thrown.
    pub fn refine_to(&self, final_knots: &KnotVec) -> Self {
        let self_knots_not_in_final = self.knots.without(final_knots);
        if self_knots_not_in_final.len() > 0 {
            panic!(
                "Cannot refine curve with knot vector {:?} to final knot vector {:?} because it contains knots that do not exist in final knot vector.", 
                self.knots,
                self_knots_not_in_final
            );
        }

        let final_knots_not_in_self = final_knots.without(&self.knots);

        self.refine_knots(final_knots_not_in_self)
    }

    /// Finds the closest point on the curve where a vector from it to `point`
    /// is perpendicular to the curve.
    pub fn project_point(&self, point: Vec3) -> Option<CurveProjectionResult> {
        let mut nearest_projected: Option<CurveProjectionResult> = None;

        let mut try_params = vec![0.0];
        for straight_bez in self.straight_beziers().iter() {
            if straight_bez.has_perpendicular_projection(point) {
                if let Some(param) = straight_bez.estimate_projection_parameter(point) {
                    try_params.push(param);
                }
            }
        }
        try_params.push(1.0);

        for i in 0..try_params.len() {
            let param = try_params[i];
            let lower_bound = if i == 0 { 0.0 } else { try_params[i - 1] };
            let upper_bound = if i == try_params.len() - 1 {
                1.0
            } else {
                try_params[i + 1]
            };

            if let Some(projected) = self.project_point_from_starting_param(
                point,
                param,
                ProjectionKind::Projection,
                (lower_bound, upper_bound),
            ) {
                if let Some(ref nearest) = nearest_projected {
                    if nearest.distance > projected.distance {
                        nearest_projected = Some(projected);
                    }
                } else {
                    nearest_projected = Some(projected);
                }
            }
        }

        nearest_projected
    }

    /// Attempts to project or invert a point onto the curve using Newton's method,
    /// starting the iterations at the parameter value `u`.
    fn project_point_from_starting_param(
        &self,
        point: Vec3,
        u: f64,
        projection_kind: ProjectionKind,
        bounds: (f64, f64),
    ) -> Option<CurveProjectionResult> {
        struct LastParams {
            u: f64,
            ders: Vec<Vec4>,
        }

        let mut last_params: Option<LastParams> = None;

        // Current parameter value that we're refining
        let mut u = u;

        for i in 0..MAX_NEWTON_ITER {
            //loop {
            // If parameter is outside of the knot vector bounds, we can't
            // project.
            if u < bounds.0 || u > bounds.1 {
                return None;
            }

            // Get position and derivatives at u
            let ders = self.eval_derivatives(u, 2);
            let point_to_pos = ders[0].project() - point;

            // If the parameter has not changed significantly since the last
            // iteration, we've converged
            if let Some(last_params) = last_params {
                if ((u - last_params.u) * last_params.ders[1]).magnitude() < TOL {
                    return Some(CurveProjectionResult {
                        u,
                        pos: ders[0],
                        distance: point_to_pos.magnitude(),
                    });
                }
            }

            // Check for convergence
            {
                // Check for point coincidence if doing inversion (does not apply to projection)
                let point_coincidence = match projection_kind {
                    // If doing inversion, check if the projected point is within tolerance of the
                    // point being inverted.
                    ProjectionKind::Inversion => point_to_pos.magnitude().toleq(0.0),
                    // If doing projection, we just mark this `true` so it has no effect
                    ProjectionKind::Projection => true,
                };

                // Check for zero cosine (within tolerance)
                let zero_cosine = {
                    let num = ders[1].project().dot(point_to_pos).abs();
                    let den = ders[1].magnitude() * point_to_pos.magnitude();
                    (num / den) <= ZERO_COS_TOL
                };

                // If points are coincident (for inversion) and the cosine is zero,
                // we've converged at u.
                if point_coincidence && zero_cosine {
                    return Some(CurveProjectionResult {
                        u,
                        pos: ders[0],
                        distance: point_to_pos.magnitude(),
                    });
                }
            }

            // Newton iteration
            let num = ders[1].project().dot(point_to_pos) * ders[0].w.powi(2);
            let den = ders[2].project().dot(point_to_pos) + ders[1].magnitude2();
            last_params = Some(LastParams { u, ders });
            u -= num / den;
        }

        None
    }

    /// Adds the given knots to the knot vector, adding and moving control
    /// points as necessary but leaving the shape of the curve intact.
    pub fn refine_knots(&self, add_knots: Vec<f64>) -> Self {
        if add_knots.len() == 0 {
            return self.clone();
        }

        let span_a = self.knots.find_span(self.degree, add_knots[0]);
        let span_b = self
            .knots
            .find_span(self.degree, add_knots[add_knots.len() - 1])
            + 1;

        let m = self.unweighted.len() + self.degree;
        let mut out_knots = vec![0.0; m + add_knots.len() + 1];
        let mut out_points = vec![Vec4::zero(); self.unweighted.len() + add_knots.len()];

        for j in 0..=span_a - self.degree {
            out_points[j] = self.weighted[j];
        }

        for j in span_b - 1..self.unweighted.len() {
            out_points[j + add_knots.len()] = self.weighted[j];
        }

        for j in 0..=span_a {
            out_knots[j] = self.knots[j];
        }

        for j in span_b + self.degree..=m {
            out_knots[j + add_knots.len()] = self.knots[j];
        }

        let mut i = span_b + self.degree - 1;
        let mut k = span_b + self.degree + add_knots.len() - 1;

        for j in (0..add_knots.len()).rev() {
            while add_knots[j] <= self.knots[i] && i > span_a {
                out_points[k - self.degree - 1] = self.weighted[i - self.degree - 1];
                out_knots[k] = self.knots[i];
                k = k - 1;
                i = i - 1;
            }

            out_points[k - self.degree - 1] = out_points[k - self.degree];

            for l in 1..=self.degree {
                let ind = k - self.degree + l;
                let mut alpha = out_knots[k + l] - add_knots[j];
                if alpha.abs() <= TOL {
                    out_points[ind - 1] = out_points[ind];
                } else {
                    alpha = alpha / (out_knots[k + l] - self.knots[i - self.degree + l]);
                    out_points[ind - 1] =
                        alpha * out_points[ind - 1] + (1.0 - alpha) * out_points[ind];
                }
            }

            out_knots[k] = add_knots[j];
            k = k - 1;
        }

        Self::create_weighted(out_points, KnotVec::new(out_knots))
    }

    /// Increases the degree of the curve to `degree`, adding control points as needed
    /// while maintaining the shape of the curve.
    pub fn elevate_degree_to(&self, degree: usize) -> Self {
        if degree < self.degree {
            panic!(
                "Tried to elevate degree {} curve to degree {}",
                self.degree, degree
            );
        }

        self.elevate_degree(degree - self.degree)
    }

    /// Increases the degree of the curve by `t`, adding control points as needed
    /// while maintaining the shape of the curve.
    pub fn elevate_degree(&self, t: usize) -> Self {
        let t: i64 = t as i64;
        let n: i64 = self.weighted.len() as i64 - 1;
        let p: i64 = self.degree as i64;
        let u = &self.knots;
        let pw = &self.weighted;
        let mut uh = vec![];
        let mut qw = vec![];

        let m: i64 = n + p + 1;
        let ph: i64 = p + t;
        let ph2: i64 = ph / 2;

        // Compute bezier degree elevation coefficients
        let mut bezalfs = vec![vec![0.0; p as usize + 1]; ph as usize + 1];
        bezalfs[0][0] = 1.0;
        bezalfs[ph as usize][p as usize] = 1.0;
        for i in 1..=ph2 {
            let inv = 1.0 / bin(ph as usize, i as usize) as f64;
            let mpi = min(p, i);

            for j in max(0, i - t)..=mpi {
                bezalfs[i as usize][j as usize] =
                    inv * bin(p as usize, j as usize) * bin(t as usize, (i - j) as usize);
            }
        }

        for i in ph2 + 1..=ph - 1 {
            let mpi = min(p, i);
            for j in max(0, i - t)..=mpi {
                bezalfs[i as usize][j as usize] = bezalfs[(ph - i) as usize][(p - j) as usize];
            }
        }

        let mut kind: i64 = ph + 1;
        let mut r: i64 = -1;
        let mut a: i64 = p;
        let mut b: i64 = p + 1;
        let mut cind: i64 = 1;
        let mut ua = u[0];
        qw.push(pw[0]);

        for _ in 0..=ph {
            uh.push(ua);
        }

        // Initialize first bezier segment
        let mut bpts = vec![Vec4::zero(); p as usize + 1];
        for i in 0..=p {
            bpts[i as usize] = pw[i as usize];
        }

        let mut alfs = vec![0.0; (p - 1) as usize];
        let mut nextbpts = vec![Vec4::zero(); (p - 1) as usize];
        let mut ebpts = vec![Vec4::zero(); (p + t + 1) as usize];
        while b < m {
            let i = b;
            while b < m && u[b as usize] == u[(b + 1) as usize] {
                b += 1;
            }

            let mul = b - i + 1;
            let ub = u[b as usize];
            let oldr = r;
            r = p - mul;

            let lbz = if oldr > 0 { (oldr + 2) / 2 } else { 1 };
            let rbz = if r > 0 { ph - (r + 1) / 2 } else { ph };

            if r > 0 {
                // Insert knot to get bezier segment
                let numer = ub - ua;
                for k in (mul + 1..=p).rev() {
                    alfs[(k - mul - 1) as usize] = numer / (u[(a + k) as usize] - ua);
                }

                for j in 1..=r {
                    let save = r - j;
                    let s = mul + j;
                    for k in (s..=p).rev() {
                        bpts[k as usize] = alfs[(k - s) as usize] * bpts[k as usize]
                            + (1.0 - alfs[(k - s) as usize]) * bpts[(k - 1) as usize];
                    }
                    nextbpts[save as usize] = bpts[p as usize];
                }
            }

            // Degree elevate bezier
            for i in lbz..=ph {
                // Only points lbz...ph are used below
                ebpts[i as usize] = Vec4::zero();
                let mpi = min(p, i);
                for j in max(0, i - t)..=mpi {
                    ebpts[i as usize] =
                        ebpts[i as usize] + (bezalfs[i as usize][j as usize] * bpts[j as usize]);
                }
            }

            if oldr > 1 {
                // Must remove knot u=u[a] oldr times
                let mut first = kind - 2;
                let mut last = kind;
                let den = ub - ua;
                let bet = (ub - uh[(kind - 1) as usize]) / den;

                // Knot removal loop
                for tr in 1..oldr {
                    let mut i = first;
                    let mut j = last;
                    let mut kj = j - kind + 1;
                    while j - i > tr {
                        // Loop and compute the new control points for one removal step
                        if i < cind {
                            let alf = (ub - uh[i as usize]) / (ua - uh[i as usize]);
                            if qw.len() <= i as usize {
                                qw.push(Vec4::zero());
                            }
                            qw[i as usize] =
                                alf * qw[i as usize] + (1.0 - alf) * qw[(i - 1) as usize];
                        }

                        if j >= lbz {
                            if j - tr <= kind - ph + oldr {
                                let gam = (ub - uh[(j - tr) as usize]) / den;
                                ebpts[kj as usize] = gam * ebpts[kj as usize]
                                    + (1.0 - gam) * ebpts[(kj + 1) as usize];
                            } else {
                                ebpts[kj as usize] = bet * ebpts[kj as usize]
                                    + (1.0 - bet) * ebpts[(kj + 1) as usize];
                            }
                        }

                        i += 1;
                        j -= 1;
                        kj -= 1;
                    }
                    first -= 1;
                    last += 1;
                }
            }

            if a != p {
                // Load the knot ua
                for _ in 0..ph - oldr {
                    uh.push(ua);
                    kind += 1;
                }
            }

            for j in lbz..=rbz {
                qw.push(ebpts[j as usize]);
                cind += 1;
            }

            if b < m {
                // Set up for next pass through loop
                for j in 0..r {
                    bpts[j as usize] = nextbpts[j as usize];
                }

                for j in r..=p {
                    bpts[j as usize] = pw[(b - p + j) as usize];
                }

                a = b;
                b += 1;
                ua = ub;
            } else {
                for _ in 0..=ph {
                    uh.push(ub);
                }
            }
        }

        Self::create_weighted(qw, KnotVec::new(uh))
    }
}

#[cfg(test)]
mod tests {
    use cgmath::vec4;

    use crate::Curve;

    #[test]
    fn identifies_convex_control_polygons() {
        let convex = Curve::create_unweighted_bezier(vec![
            vec4(-2.0, 1.0, 0.0, 1.0),
            vec4(-1.0, -1.0, 0.0, 1.0),
            vec4(1.0, -1.0, 0.0, 1.0),
            vec4(2.0, 1.0, 0.0, 1.0),
        ]);

        assert!(convex.is_convex());

        let concave = Curve::create_unweighted_bezier(vec![
            vec4(-2.0, 1.0, 0.0, 1.0),
            vec4(-1.0, -1.0, 0.0, 1.0),
            vec4(1.0, 0.9, 0.0, 1.0),
            vec4(2.0, 1.0, 0.0, 1.0),
        ]);

        assert!(!concave.is_convex());

        let complex = Curve::create_unweighted_bezier(vec![
            vec4(-2.0, 1.0, 0.0, 1.0),
            vec4(-1.0, -1.0, 0.0, 1.0),
            vec4(1.0, 1.5, 0.0, 1.0),
            vec4(2.0, 1.0, 0.0, 1.0),
        ]);

        assert!(!complex.is_convex());
    }
}
