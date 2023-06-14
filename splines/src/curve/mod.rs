mod builders;

use std::cmp::{max, min};

use crate::{basis, knot_span, normalize_knots, HPoint, Pt4};
use cgmath::{Matrix4, Zero};

pub use builders::*;

#[derive(Debug, Clone)]
pub struct Curve {
    pub(crate) weighted: Vec<Pt4>,
    pub(crate) unweighted: Vec<Pt4>,
    pub(crate) knots: Vec<f64>,
    pub(crate) order: usize,
    pub(crate) degree: usize,
}
impl Curve {
    pub fn new(unweighted: Vec<Pt4>, knots: Vec<f64>) -> Self {
        let weighted = unweighted.iter().map(HPoint::weight).collect();
        Self::create(unweighted, weighted, knots)
    }

    pub fn weighted(weighted: Vec<Pt4>, knots: Vec<f64>) -> Self {
        let unweighted = weighted.iter().map(HPoint::unweight).collect();
        Self::create(unweighted, weighted, knots)
    }

    fn create(unweighted: Vec<Pt4>, weighted: Vec<Pt4>, knots: Vec<f64>) -> Self {
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

        let knots = normalize_knots(knots);

        for i in 0..order {
            if knots[i] != 0.0 {
                panic!(
                    "Unclamped curve: First {} normalized knots must be 0, but knots are {:?}",
                    order, knots
                );
            }
        }

        for i in 0..order {
            if knots[knots.len() - i - 1] != 1.0 {
                panic!(
                    "Unclamped curve: Last {} normalized knots must be 1, but knots are {:?}",
                    order, knots
                );
            }
        }

        Self {
            weighted,
            unweighted,
            knots,
            order,
            degree,
        }
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

    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    pub fn eval(&self, t: f64) -> Pt4 {
        self.weighted
            .iter()
            .enumerate()
            .map(|(j, p)| p * basis(&self.knots, j, self.order, t))
            .sum()
    }

    pub fn transform(&self, transform: &Matrix4<f64>) -> Self {
        Self::new(
            self.unweighted
                .iter()
                .map(|p| p.transform(transform))
                .collect(),
            self.knots.clone(),
        )
    }

    pub fn refine_knots(&self, add_knots: Vec<f64>) -> Self {
        let span_a = knot_span(&self.knots, self.unweighted.len(), add_knots[0]);
        let span_b = knot_span(
            &self.knots,
            self.unweighted.len(),
            add_knots[add_knots.len() - 1],
        ) + 1;

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
                if alpha.abs() == 0.0 {
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

        Self::weighted(out_points, out_knots)
    }

    pub fn elevate_degree(&self, num_elevations: usize) -> Self {
        todo!()
        /*
        let n = self.points.len() as i64;
        let p = self.degree as i64;
        let U = &self.knots;
        let Pw = &self.points;
        let t = num_elevations as i64;
        let mut nh: usize = 0;
        let mut Uh = vec![0.0; self.knots.len() + num_elevations];
        let mut Qw = vec![Pt4::zero(); todo!()];

        let m = n + p + 1;
        let ph = p + t;
        let ph2 = ph / 2;

        let mut bezalfs = vec![vec![0.0; p as usize + 1]; p as usize + t as usize + 1];
        let mut bpts = vec![Pt4::zero(); p as usize + 1];
        let mut ebpts = vec![Pt4::zero(); p as usize + t as usize + 1];
        let mut Nextbpts = vec![Pt4::zero(); p as usize - 1];
        let mut alphas = vec![0.0; p as usize - 1];

        bezalfs[0][0] = 1.0;
        bezalfs[ph as usize][p as usize] = 1.0;

        for i in 1..=ph2 {
            let inv = 1.0 / bin(ph as f64, i as f64);
            let mpi = min(p, i);
            for j in max(0, i - t)..=mpi {
                bezalfs[i as usize][j as usize] =
                    inv * bin(p as f64, j as f64) * bin(t as f64, (i - j) as f64);
            }
        }

        for i in ph2 + 1..=ph - 1 {
            let mpi = p.min(i);
            for j in max(0, i - t)..=mpi {
                bezalfs[i as usize][j as usize] = bezalfs[(ph - i) as usize][(p - j) as usize];
            }
        }

        let mh = ph;
        let mut kind = ph + 1;
        let r: i64 = -1;
        let a = p;
        let mut b = p + 1;
        let cind = 1;
        let ua = U[0];

        for i in 0..=ph {
            Uh[i as usize] = ua;
        }

        for i in 0..=p {
            bpts[i as usize] = Pw[i as usize];
        }

        while b < m {
            let i = b;
            while b < m && U[b as usize] == U[(b + 1) as usize] {
                b = b + 1;
            }

            let mul = b - i + 1;
            let mh = mh + mul + t;
            let ub = U[b as usize];
            let oldr = r;
            let r = p - mul;

            let lbz: i64 = if oldr > 0 { (oldr + 2) / 2 } else { 1 };

            let rbz = if r > 0 { ph - (r + 1) / 2 } else { ph };

            if r > 0 {
                // Insert knot to get bezier segment
                let numer = ub - ua;
                let mut alfs = vec![0.0; (p - mul) as usize];
                for k in (mul + 1..=p).rev() {
                    alfs[(k - mul - 1) as usize] = numer / (U[(a + k) as usize] - ua);
                }

                for j in 1..=r {
                    let save = r - j;
                    let s = mul + j;
                    for k in (s..=p).rev() {
                        bpts[k as usize] = alfs[(k - s) as usize] * bpts[k as usize]
                            + (1.0 - alfs[(k - s) as usize]) * bpts[(k - 1) as usize];
                    }
                    Nextbpts[save as usize] = bpts[p as usize];
                }
            }

            for i in lbz..=ph {
                // Degree elevate bezier
                ebpts[i as usize] = Pt4::zero(); // 0.0 ?
                let mpi = min(p, i);
                for j in max(0, i - t)..=mpi {
                    ebpts[i as usize] =
                        ebpts[i as usize] + bezalfs[i as usize][j as usize] * bpts[j as usize];
                }
            }

            if oldr > 1 {
                // Must remove knot u = U[a] oldr times
                let mut first = kind - 2;
                let mut last = kind;
                let den = ub - ua;
                let bet = (ub - Uh[(kind - 1) as usize]) / den;

                for tr in 1..oldr {
                    // Knot removal loop
                    let mut i = first;
                    let mut j = last;
                    let mut kj = j - kind + 1;

                    while j - i > tr {
                        // Loop and compute the control points
                        // for one removal step
                        if i < cind {
                            let alf = (ub - Uh[i as usize]) / (ua - Uh[i as usize]);
                            Qw[i as usize] =
                                alf * Qw[i as usize] + (1.0 - alf) * Qw[(i - 1) as usize];
                        }

                        if j >= lbz {
                            if j - tr <= kind - ph + oldr {
                                let gam = (ub - Uh[(j - tr) as usize]) / den;
                                ebpts[kj as usize] = gam * ebpts[kj as usize]
                                    + (1.0 - gam) * ebpts[(kj + 1) as usize];
                            } else {
                                ebpts[kj as usize] = bet * ebpts[kj as usize]
                                    + (1.0 - bet) * ebpts[(kj + 1) as usize];
                            }
                        }

                        i = i + 1;
                        j = j - 1;
                        kj = kj - 1;
                    }

                    first = first - 1;
                    last = last + 1;
                }
            } // End of removing knot, u=U[a]

            if a != p {
                // Load the knot ua
                for i in 0..ph - oldr {
                    Uh[kind as usize] = ua;
                    kind = kind + 1;
                }
            }

            for j in lbz..=rbz {
                // Load control points into Qw
                Qw[cind as usize] = ebpts[j as usize];
                cind = cind + 1;
            }

            if b < m {
                // Set up for next pass through loop
                for j in 0..r {
                    bpts[j as usize] = Nextbpts[j as usize];
                }

                for j in r..=p {
                    bpts[j as usize] = Pw[(b - p + j) as usize];
                }

                a = b;
                b = b + 1;
                ua = ub;
            } else {
                // End knot
                for i in 0..=ph {
                    Uh[(kind + 1) as usize] = ub;
                }
            }
        }

        nh = (mh - ph - 1) as usize;

        Self::new(Qw, Uh)
        */
    }
}

fn bin(a: f64, b: f64) -> f64 {
    todo!()
}
