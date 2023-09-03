mod bezier;
mod builders;

use crate::{
    basis, bin, curve_derivatives, knots::KnotVec, line_to_point_perpendicular, nurbs_to_beziers,
    refine_knots, Vec4,
};
use cgmath::{InnerSpace, Matrix4, Zero};
use once_cell::unsync::OnceCell;
use primitives::{EVec, HVec, TolEq, Vec3, TOL};
use std::cmp::{max, min};

pub use bezier::*;
pub use builders::*;

const MAX_NEWTON_ITER: usize = 2000;
const ZERO_COS_TOL: f64 = TOL / 100.0;

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
    beziers: OnceCell<Vec<CurveBezierComponent>>,
    convex_beziers: OnceCell<Vec<CurveBezierComponent>>,
    straight_beziers: OnceCell<Vec<CurveBezierComponent>>,
    endpoints: OnceCell<(Vec4, Vec4)>,
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
            endpoints: OnceCell::new(),
        }
    }

    pub fn endpoints(&self) -> &(Vec4, Vec4) {
        self.endpoints
            .get_or_init(|| (self.eval_pos(0.0), self.eval_pos(1.0)))
    }

    /// Returns the piecewise bezier decomposition of this curve.
    pub fn beziers(&self) -> &[CurveBezierComponent] {
        self.beziers
            .get_or_init(|| nurbs_to_beziers(&self.weighted, self.degree, &self.knots))
    }

    /// Returns the piecewise bezier decomposition of this curve, subdivided
    /// until each bezier curve has a convex control polygon.
    pub fn convex_beziers(&self) -> &[CurveBezierComponent] {
        self.convex_beziers.get_or_init(|| {
            let mut convex_beziers = vec![];

            for bezier in self.beziers().iter() {
                convex_beziers.extend(bezier.split_until_convex());
            }

            convex_beziers
        })
    }

    pub fn straight_beziers(&self) -> &[CurveBezierComponent] {
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

    fn get_projection_try_params(&self, point: Vec3) -> Vec<f64> {
        let mut try_params = vec![];
        for straight_bez in self.straight_beziers().iter() {
            if straight_bez.has_perpendicular_projection(point) {
                if let Some(param) = straight_bez.estimate_projection_parameter(point) {
                    try_params.push(param);
                }
            }
        }

        try_params
    }

    /// Finds the closest point on the curve to `point`.
    pub fn nearest_point(&self, point: Vec3) -> CurveProjectionResult {
        //let mut nearest_projected: Option<CurveProjectionResult> = None;

        // If the closest point on the curve isn't one of the projected points we'll
        // search for below, it's one of the endpoints. We start by finding the distance
        // to the starting and ending points and setting `nearest_projected` to the closest
        // one.
        let (start_point, end_point) = self.endpoints();

        let start_dist = (start_point.project() - point).magnitude();
        let end_dist = (end_point.project() - point).magnitude();

        let nearest = match start_dist < end_dist {
            true => CurveProjectionResult {
                u: 0.0,
                pos: start_point.clone(),
                distance: start_dist,
            },
            false => CurveProjectionResult {
                u: 1.0,
                pos: end_point.clone(),
                distance: end_dist,
            },
        };

        if let Some(projected) = self.project_point(point) {
            if projected.distance < nearest.distance {
                projected
            } else {
                nearest
            }
        } else {
            nearest
        }
    }

    /// Finds the closest point on the curve where a vector from it to `point`
    /// is perpendicular to the curve.
    pub fn project_point(&self, point: Vec3) -> Option<CurveProjectionResult> {
        let mut nearest_projected: Option<CurveProjectionResult> = None;

        for param in self.get_projection_try_params(point).into_iter() {
            if let Some(projected) = self.project_point_from_starting_param(point, param) {
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

    /// Finds the point on the curve that is in the same position as `point` (within tolerance)
    pub fn invert_point(&self, point: Vec3) -> Option<CurveProjectionResult> {
        let mut nearest_projected: Option<CurveProjectionResult> = None;

        for param in self.get_projection_try_params(point).into_iter() {
            if let Some(projected) = self.project_point_from_starting_param(point, param) {
                if let Some(ref nearest) = nearest_projected {
                    if nearest.distance > projected.distance {
                        nearest_projected = Some(projected);
                    }
                } else {
                    nearest_projected = Some(projected);
                }
            }
        }

        if let Some(ref projected) = nearest_projected {
            if projected.distance.toleq(0.0) {
                nearest_projected
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Attempts to project or invert a point onto the curve using Newton's method,
    /// starting the iterations at the parameter value `u`.
    fn project_point_from_starting_param(
        &self,
        point: Vec3,
        u: f64,
    ) -> Option<CurveProjectionResult> {
        struct LastParams {
            u: f64,
            ders: Vec<Vec4>,
        }

        let mut last_params: Option<LastParams> = None;

        // Current parameter value that we're refining
        let mut u = u;

        for _ in 0..MAX_NEWTON_ITER {
            if u < 0.0 || u > 1.0 {
                return None;
            }

            // Get position and derivatives at u
            let ders = self.eval_derivatives(u, 2);
            let point_to_pos = ders[0].project() - point;

            // If the parameter has not changed significantly since the last
            // iteration, we've converged
            if let Some(last_params) = last_params {
                if ((u - last_params.u) * last_params.ders[1]).magnitude() < TOL / 100.0 {
                    return Some(CurveProjectionResult {
                        u,
                        pos: ders[0],
                        distance: point_to_pos.magnitude(),
                    });
                }
            }

            // More stopping conditions
            {
                let zero_cosine = {
                    let num = ders[1]
                        .project()
                        .normalize()
                        .dot(point_to_pos.normalize())
                        .abs();
                    let den = ders[1].magnitude() * point_to_pos.magnitude();
                    (num / den) <= ZERO_COS_TOL
                };

                let point_coincidence = point_to_pos.magnitude().toleq(0.0);

                if zero_cosine || point_coincidence {
                    return Some(CurveProjectionResult {
                        u,
                        pos: ders[0],
                        distance: point_to_pos.magnitude(),
                    });
                }
            }

            // Newton iteration
            let num = ders[1].project().normalize().dot(point_to_pos);
            let den = ders[2].project().normalize().dot(point_to_pos) + ders[1].magnitude2();
            last_params = Some(LastParams { u, ders });
            u -= num / den;
        }

        None
    }

    /// Adds the given knots to the knot vector, adding and moving control
    /// points as necessary but leaving the shape of the curve intact.
    pub fn refine_knots(&self, add_knots: Vec<f64>) -> Self {
        let (weighted, knots) = refine_knots(self.degree, &self.knots, &self.weighted, &add_knots);
        Self::create_weighted(weighted, knots)
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
    use cgmath::{vec3, vec4};
    use primitives::HVec;

    use crate::{Curve, CurveBezierComponent};

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
