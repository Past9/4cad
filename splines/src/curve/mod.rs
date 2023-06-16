mod builders;

use core::num;
use std::cmp::{min, max};

use crate::{basis, knots::KnotVec, HPoint, Pt4, TOL, bin};
use cgmath::{Matrix4, Zero};

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
    pub fn new(unweighted: Vec<Pt4>, knots: KnotVec) -> Self {
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

    pub fn elevate_degree(&self, num_elevations: usize) -> Self {
        println!("elevate_degree for {:?} (num_elevations = {})", self, num_elevations);
        // Decompose into beziers
        let beziers = self.decompose();

        println!("beziers {:?}", beziers);

        // Degree elevate each bezier
        let mut elevated_beziers = Vec::new();
        for bezier in beziers.iter() {
            let mut elevated_points = vec![Pt4::zero(); self.degree + 1 + num_elevations];
            for i in 0..elevated_points.len() {
                let start = max(0, i as i64 - num_elevations  as i64) as usize;
                let end = min(self.degree, i);
                for j in start..=end {
                    let coeff = (bin(self.degree, j) * bin(num_elevations, i - j)) / bin(self.degree + num_elevations, i);
                    elevated_points[i] += coeff * bezier[j];
                }
            }
            elevated_beziers.push(elevated_points);
        }

        println!("elevated_beziers {:?}", elevated_beziers);

        // Combine the elevated beziers back into a single curve
        let mut new_weighted = vec![];
        for i in 0..elevated_beziers.len() - 1 {
            for p in 0..elevated_beziers[i].len() - 1 {
                new_weighted.push(elevated_beziers[i][p]);
            }
        }
        new_weighted.extend(elevated_beziers.last().unwrap().into_iter().map(|pt| *pt));

        println!("new_weighted {:?}", new_weighted);

        // Generate uniform knots
        let new_degree = self.degree + 1;
        let num_total_knots = new_weighted.len() + new_degree + 1;
        let num_clamp_knots = new_degree + 1;
        let num_middle_knots = num_total_knots - num_clamp_knots * 2;
        let mut new_knots = vec![0.0; num_clamp_knots];
        for i in 1..=num_middle_knots {
            new_knots.push(i as f64);
        }
        new_knots.extend(vec![(num_middle_knots + 1) as f64; num_clamp_knots]);

        println!("new_knots {:?}", new_knots);

        Self::new(new_weighted, KnotVec::new(new_knots))
    }

    /// Decomposes the NURBS curve into a series of bezier segments. Returns
    /// a `Vec` of each segment's control points.
    fn decompose(&self) -> Vec<Vec<Pt4>> {
        let m = self.weighted.len() + self.degree;
        let mut a = self.degree;
        let mut b = self.degree + 1;
        let mut nb = 0;
    
        let new_bezier_points = vec![Pt4::zero(); self.degree + 1];
        let mut bezier_ctrl_pts: Vec<Vec<Pt4>> = Vec::new();
    
        bezier_ctrl_pts.push(new_bezier_points.clone());
    
        for i in 0..=self.degree {
            bezier_ctrl_pts[nb][i] = self.weighted[i];
        }
    
        while b < m {
            let i = b;
            while b < m && self.knots[b + 1] == self.knots[b] {
                b += 1;
            }
    
            let mult = b - i + 1;
            if mult < self.degree {
                let numer = self.knots[b] - self.knots[a];
                let mut alphas = vec![0.0; self.degree - mult];
                for j in ((mult + 1)..=self.degree).rev() {
                    alphas[j - mult - 1] = numer / (self.knots[a + j] - self.knots[a]);
                }
    
                let r = self.degree - mult;
                for j in 1..=r {
                    let save = r - j;
                    let s = mult + j;
                    for k in (s..=self.degree).rev() {
                        let alpha = alphas[k - s];
                        bezier_ctrl_pts[nb][k] =
                            bezier_ctrl_pts[nb][k] * alpha + bezier_ctrl_pts[nb][k - 1] * (1.0 - alpha);
                    }
    
                    if b < m {
                        if bezier_ctrl_pts.len() - 1 < nb + 1 {
                            bezier_ctrl_pts.push(new_bezier_points.clone());
                        }
                        bezier_ctrl_pts[nb + 1][save] = bezier_ctrl_pts[nb][self.degree];
                    }
                }
            }
    
            nb += 1;
    
            if b < m {
                for i in (self.degree - mult)..=self.degree {
                    if bezier_ctrl_pts.len() - 1 < nb {
                        bezier_ctrl_pts.push(new_bezier_points.clone());
                    }
                    bezier_ctrl_pts[nb][i] = self.weighted[b - self.degree + i];
                }
                a = b;
                b += 1;
            }
        }
    
        bezier_ctrl_pts
    }
}

