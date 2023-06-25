use std::ops::Index;

use crate::TolEq;

#[derive(Debug, Clone)]
pub struct KnotVec {
    knots: Vec<f64>,
}
impl KnotVec {
    pub fn new(knots: Vec<f64>) -> Self {
        Self {
            knots: Self::normalize_knots(knots),
        }
    }

    pub fn uniform(num_points: usize, degree: usize) -> Self {
        let num_total_knots = num_points + degree + 1;
        let num_clamp_knots = degree + 1;
        let num_middle_knots = num_total_knots - num_clamp_knots * 2;
        let mut new_knots = vec![0.0; num_clamp_knots];
        for i in 1..=num_middle_knots {
            new_knots.push(i as f64);
        }
        new_knots.extend(vec![(num_middle_knots + 1) as f64; num_clamp_knots]);

        Self::new(new_knots)
    }

    pub fn from<const N: usize>(knots: [f64; N]) -> Self {
        Self::new(knots.to_vec())
    }

    pub fn take_knots(self) -> Vec<f64> {
        self.knots
    }

    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    pub fn len(&self) -> usize {
        self.knots.len()
    }

    pub fn assert_clamped(&self, degree: usize) {
        self.assert_clamped_start(degree);
        self.assert_clamped_end(degree);
    }

    pub fn assert_clamped_start(&self, degree: usize) {
        if !self.is_clamped_start(degree) {
            panic!(
                "Unclamped spline: First {} normalized knots must be 0.0, but knots are {:?}",
                degree + 1,
                self
            );
        }
    }

    pub fn assert_clamped_end(&self, degree: usize) {
        if !self.is_clamped_end(degree) {
            panic!(
                "Unclamped spline: Last {} normalized knots must be 1.0, but knots are {:?}",
                degree + 1,
                self
            );
        }
    }

    pub fn is_clamped(&self, degree: usize) -> bool {
        self.is_clamped_start(degree) && self.is_clamped_end(degree)
    }

    pub fn is_clamped_start(&self, degree: usize) -> bool {
        for i in 0..=degree {
            if !self[i].toleq(0.0) {
                return false;
            }
        }

        true
    }

    pub fn is_clamped_end(&self, degree: usize) -> bool {
        for i in 0..=degree {
            if !self[self.len() - i - 1].toleq(1.0) {
                return false;
            }
        }

        true
    }

    pub fn find_span(&self, degree: usize, pos: f64) -> usize {
        // Alg 2.1
        let num_pts = self.knots.len() - degree - 1;

        if pos == self[num_pts] {
            return num_pts - 1;
        }

        let mut low = degree;
        let mut high = num_pts + 1;
        let mut mid = (low + high) / 2;

        while pos < self[mid] || pos >= self[mid + 1] {
            if pos < self[mid] {
                high = mid;
            } else {
                low = mid;
            }

            mid = (low + high) / 2;
        }

        return mid;
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = vec![];

        let mut i_self = 0;
        let mut i_other = 0;
        while i_self < self.len() || i_other < other.len() {
            let k_self = self[i_self];
            let k_other = other[i_other];

            if let Some(avg) = k_self.toleq_avg(k_other) {
                merged.push(avg);
                i_self += 1;
                i_other += 1;
            } else if k_self < k_other {
                merged.push(k_self);
                i_self += 1;
            } else if k_other < k_self {
                merged.push(k_other);
                i_other += 1;
            }
        }

        Self::new(merged)
    }

    /// Outputs a list of knots that are in `self` but not in `remove`. Knot
    /// eqality is tested within tolerance. Each knot in `remove_knots` will
    /// be removed at most once, so to remove N copies of a knot, it
    /// must be included in `remove_knots` N times.
    pub fn without(&self, remove: &KnotVec) -> Vec<f64> {
        let mut out_knots = vec![];

        let mut i_self = 0;
        let mut i_remove = 0;
        while i_self < self.len() && i_remove < remove.len() {
            let k_self = self[i_self];
            let k_remove = remove[i_remove];

            if k_self.toleq(k_remove) {
                i_self += 1;
                i_remove += 1;
            } else if k_self < k_remove {
                out_knots.push(k_self);
                i_self += 1;
            } else if k_remove < k_self {
                i_remove += 1;
            }
        }

        while i_self < self.len() {
            out_knots.push(self[i_self]);
            i_self += 1;
        }

        out_knots
    }

    fn normalize_knots(knots: Vec<f64>) -> Vec<f64> {
        if knots.len() == 0 {
            return knots;
        }

        let max_knot = knots[knots.len() - 1].clone();
        knots.into_iter().map(|knot| knot / &max_knot).collect()
    }
}
impl Index<usize> for KnotVec {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.knots[index]
    }
}
impl TolEq for KnotVec {
    fn toleq(self, rhs: Self) -> bool {
        self.knots.toleq(rhs.knots)
    }

    fn toleq_avg(self, rhs: Self) -> Option<Self>
    where
        Self: Sized,
    {
        self.knots.toleq_avg(rhs.knots).map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use crate::{knots::KnotVec, TolEq};

    #[test]
    fn merges_knots() {
        let merged =
            KnotVec::from([0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 4.0, 4.0, 4.0]).merge(&KnotVec::from([
                0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0,
            ]));

        assert!(merged.toleq(KnotVec::from([
            0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 4.0
        ])));
    }

    #[test]
    fn removes_knots() {
        let without = KnotVec::from([0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 4.0, 4.0, 4.0])
            .without(&KnotVec::from([0.0, 1.0, 4.0, 4.0]));
        assert!(without.toleq(KnotVec::from([0.0, 0.0, 2.0, 2.0, 4.0]).take_knots()));

        let without = KnotVec::from([0.0, 1.0, 3.0, 3.1, 3.2, 4.0, 4.0]).without(&KnotVec::from([
            0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 4.0, 4.0, 4.0,
        ]));

        assert!(without.toleq(vec![0.75, 0.775, 0.8]));
    }
}
