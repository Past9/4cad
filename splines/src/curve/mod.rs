mod builders;

use crate::{
    basis, basis_derivatives, bin, curve_derivatives, knots::KnotVec, EPoint, HPoint, Pt3, Pt4,
    Vec3, TOL,
};
use cgmath::{EuclideanSpace, Matrix4, Vector3, Vector4, Zero};
use core::num;
use std::cmp::{max, min};

pub use builders::*;

#[derive(Debug, Clone)]
pub struct Curve {
    pub(crate) weighted: Vec<Pt4>,
    pub(crate) unweighted: Vec<Pt4>,
    pub(crate) knots: KnotVec,
    pub(crate) order: usize,
    pub(crate) degree: usize,
}
impl Curve {
    pub fn unweighted(unweighted: Vec<Pt4>, knots: KnotVec) -> Self {
        let weighted = unweighted.iter().map(HPoint::weight).collect();
        Self::create(unweighted, weighted, knots)
    }

    pub fn weighted(weighted: Vec<Pt4>, knots: KnotVec) -> Self {
        let unweighted = weighted.iter().map(HPoint::unweight).collect();
        Self::create(unweighted, weighted, knots)
    }

    fn create(unweighted: Vec<Pt4>, weighted: Vec<Pt4>, knots: KnotVec) -> Self {
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
        }
    }

    pub fn take_weighted(self) -> Vec<Pt4> {
        self.weighted
    }

    pub fn take_unweighted(self) -> Vec<Pt4> {
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

    pub fn eval(&self, u: f64) -> Pt4 {
        // Alg A4.1
        let span = self.knots.find_span(self.degree, u);
        let basis = basis(span, u, self.degree, &self.knots);
        let mut point = Pt4::zero();
        for j in 0..=self.degree {
            point += basis[j] * self.weighted[span - self.degree + j];
        }

        point
    }

    pub fn eval_derivatives(&self, u: f64, num_derivatives: usize) -> Vec<Vector4<f64>> {
        let homogeneous_derivatives =
            curve_derivatives(u, &self.weighted, self.degree, &self.knots, num_derivatives);

        let mut derivatives = vec![Vector4::zero(); num_derivatives + 1];

        for k in 0..=num_derivatives {
            let mut pt3 = homogeneous_derivatives[k].truncate();
            for i in 1..=k {
                pt3 -= derivatives[k - i].truncate() * bin(k, i) * homogeneous_derivatives[i].w;
            }
            derivatives[k] = (Pt3::origin() + pt3).to_hpoint(homogeneous_derivatives[0].w);
            // / homogeneous_derivatives[0].w;
        }

        derivatives
    }

    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::unweighted(
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
        let mut out_points = vec![Pt4::zero(); self.unweighted.len() + add_knots.len()];

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

        Self::weighted(out_points, KnotVec::new(out_knots))
    }

    pub fn elevate_degree_to(&self, degree: usize) -> Self {
        if degree < self.degree {
            panic!(
                "Tried to elevate degree {} curve to degree {}",
                self.degree, degree
            );
        }

        self.elevate_degree(degree - self.degree)
    }

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
        let mut bpts = vec![Pt4::zero(); p as usize + 1];
        for i in 0..=p {
            bpts[i as usize] = pw[i as usize];
        }

        let mut alfs = vec![0.0; (p - 1) as usize];
        let mut nextbpts = vec![Pt4::zero(); (p - 1) as usize];
        let mut ebpts = vec![Pt4::zero(); (p + t + 1) as usize];
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
                ebpts[i as usize] = Pt4::zero();
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
                                qw.push(Pt4::zero());
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

        Self::weighted(qw, KnotVec::new(uh))
    }
}
