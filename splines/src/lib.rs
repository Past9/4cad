use primitives::{EPoint, HPoint, Param, Rat};

pub struct Curve {
    points: Vec<HPoint>,
    knots: Vec<Rat>,
}
impl Curve {
    pub fn new(points: Vec<HPoint>, knots: Vec<Rat>) -> Self {
        let k = knots.len();
        let n = points.len();
        let m = k - n - 1;
        if m < 1 {
            panic!(
                "Curve would have degree {} (knots.len() - points.len() - 1). Needs more knots or fewer points.",
                m
            );
        }

        Self { points, knots }
    }

    fn knot_span(&self, m: usize, n: usize, t: &Rat) -> usize {
        if t == &self.knots[n] {
            return n - 1;
        }

        let mut low = m;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;

        while t < &self.knots[mid] || t >= &self.knots[mid + 1] {
            if t < &self.knots[mid] {
                high = mid;
            } else {
                low = mid;
            }

            mid = (low + high) / 2;
        }

        return mid;
    }

    pub fn eval_s(&self, t: &Rat) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis_s(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }

    /*
    pub fn eval_d1(&self, t: &Rat) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis_d1(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }

    pub fn eval_d2(&self, t: &Param) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis_d2(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }
    */

    fn basis_s(&self, j: usize, m: usize, t: &Rat) -> Rat {
        let degree = self.degree();
        let n = self.points.len();
        let i = self.knot_span(degree, n, t);

        if m == 1 {
            let i = self.knot_span(degree, n, t);

            println!("i @ {} = {}", t, i);

            let ti = &self.knots[i];
            let ti1 = &self.knots[i + 1];

            if ti <= &t && t < ti1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let tj = &self.knots[j];
            let tj1 = &self.knots[j + 1];
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            println!("A {} {} {}", tjmsub1, tj, tjmsub1 - tj);
            println!("B {} {} {}", tjm, tj1, tjm - tj1);

            ((t - tj) / (tjmsub1 - tj)) * self.basis_s(j, m - 1, t)
                + ((tjm - t) / (tjm - tj1)) * self.basis_s(j + 1, m - 1, t)
        }
    }

    /*
    fn basis_d1(&self, j: usize, m: usize, t: &Rat) -> Rat {
        if m == 1 {
            let ti = &self.knots[j];
            let ti1 = &self.knots[j + 1];

            if ti <= &t && t < ti1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let h = t.num();
            let s = t.den();
            let tj = &self.knots[j];
            let tj1 = &self.knots[j + 1];
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            ((s - h * tj) / (tjmsub1 - tj)) * self.basis_d1(j, m - 1, t)
                + ((h * tjm - s) / (tjm - tj1)) * self.basis_d1(j + 1, m - 1, t)
        }
    }

    fn basis_d2(&self, j: usize, m: usize, t: &Param) -> Rat {
        let n = t.n();
        let o = t.o();

        if m == 1 {
            let ti = &self.knots[j];
            let ti1 = &self.knots[j + 1];

            let rat_param = Rat::new(o, n + o);

            if ti <= &rat_param && rat_param < *ti1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let tj = &self.knots[j];
            let tj1 = &self.knots[j + 1];
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            ((o - (n + o) * tj) / (tjmsub1 - tj)) * self.basis_d2(j, m - 1, t)
                + (((n + o) * tjm - o) / (tjm - tj1)) * self.basis_d2(j + 1, m - 1, t)
        }
    }
    */

    pub fn degree(&self) -> usize {
        self.knots.len() - self.points.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use primitives::{rat, EPoint, Param};

    use crate::Curve;

    #[test]
    pub fn arc() {
        let curve = Curve::new(
            vec![
                EPoint::new_ints(0, -2, 0).homogenize_int(2),
                EPoint::new_ints(1, -1, 0).homogenize_int(1),
                EPoint::new_ints(1, 1, 0).homogenize_int(1),
                EPoint::new_ints(0, 2, 0).homogenize_int(2),
                EPoint::new_ints(-1, 1, 0).homogenize_int(1),
                EPoint::new_ints(-1, -1, 0).homogenize_int(1),
                EPoint::new_ints(0, -2, 0).homogenize_int(2),
            ],
            vec![
                0.into(),
                0.into(),
                0.into(),
                1.into(),
                2.into(),
                2.into(),
                3.into(),
                4.into(),
                4.into(),
                4.into(),
            ],
        );

        let num_pts = 10;
        for i in 0..=num_pts {
            let t = rat(i, num_pts) * 4;
            println!("t = {}", t);
            println!("{} @ t = {}", curve.eval_s(&t), t);
        }
    }
}
