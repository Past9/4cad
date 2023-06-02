use primitives::rat;
use primitives::ParamD2;
use primitives::Point4d;
use primitives::Rat;

#[derive(Debug)]
pub struct Curve {
    points: Vec<Point4d>,
    knots: Vec<Rat>,
}
impl Curve {
    pub fn new(points: Vec<Point4d>, knots: Vec<Rat>) -> Self {
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

    pub fn max_knot(&self) -> Rat {
        self.knots[self.knots.len() - 1].clone()
    }

    pub fn knot_span(&self, pos: &Rat) -> usize {
        let degree = self.degree();
        let num_pts = self.points.len();
        if *pos == self.knots[num_pts] {
            return num_pts - 1;
        }

        let mut low = degree;
        let mut high = num_pts + 1;
        let mut mid = (low + high) / 2;

        while *pos < self.knots[mid] || *pos >= self.knots[mid + 1] {
            if *pos < self.knots[mid] {
                high = mid;
            } else {
                low = mid;
            }

            mid = (low + high) / 2;
        }

        return mid;
    }

    pub fn eval_s(&self, t: &Rat) -> Point4d {
        let m = self.order();
        self.points
            .iter()
            .enumerate()
            .map(|(j, p)| p * self.basis_s(j, m, &t))
            .sum()
    }

    pub fn eval_d1(&self, t: &Rat) -> Point4d {
        let m = self.order();
        (0..self.points.len())
            .map(|j| self.basis_d1(j, m, &t) * &self.points[j])
            .sum()
    }

    pub fn eval_d2(&self, t: &ParamD2) -> Point4d {
        let m = self.order();
        (0..self.points.len())
            .map(|j| self.basis_d2(j, m, &t) * &self.points[j])
            .sum()
    }

    fn basis_s(&self, j: usize, m: usize, t: &Rat) -> Rat {
        let tj = &self.knots[j];
        let tj1 = &self.knots[j + 1];

        if m == 1 {
            if tj <= &t && t < tj1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            let den1 = tjmsub1 - tj;
            let l = if den1.is_zero() {
                0.into()
            } else {
                ((t - tj) / den1) * self.basis_s(j, m - 1, t)
            };

            let den2 = tjm - tj1;
            let r = if den2.is_zero() {
                0.into()
            } else {
                ((tjm - t) / den2) * self.basis_s(j + 1, m - 1, t)
            };

            l + r
        }
    }

    fn basis_d1(&self, j: usize, m: usize, t: &Rat) -> Rat {
        let tj = &self.knots[j];
        let tj1 = &self.knots[j + 1];

        if m == 1 {
            if tj <= &t && t < tj1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let s = t.num();
            let h = t.den();
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            let den1 = tjmsub1 - tj;
            let l = if den1.is_zero() {
                0.into()
            } else {
                ((s - h * tj) / den1) * self.basis_d1(j, m - 1, t)
            };

            let den2 = tjm - tj1;
            let r = if den2.is_zero() {
                0.into()
            } else {
                ((h * tjm - s) / den2) * self.basis_d1(j + 1, m - 1, t)
            };

            l + r
        }
    }

    fn basis_d2(&self, j: usize, m: usize, t: &ParamD2) -> Rat {
        let tj = &self.knots[j];
        let tj1 = &self.knots[j + 1];

        let n = t.n();
        let o = t.o();
        let rat_param = rat(o, n + o);

        println!("RP {}", rat_param);

        if m == 1 {
            if tj <= &rat_param && &rat_param < tj1 {
                Rat::one()
            } else {
                Rat::zero()
            }
        } else {
            let tjm = &self.knots[j + m];
            let tjmsub1 = &self.knots[j + m - 1];

            let den1 = tjmsub1 - tj;
            let l = if den1.is_zero() {
                0.into()
            } else {
                ((o - (n + o) * tj) / den1) * self.basis_d2(j, m - 1, t)
            };

            let den2 = tjm - tj1;
            let r = if den2.is_zero() {
                0.into()
            } else {
                (((n + o) * tjm - o) / den2) * self.basis_d2(j + 1, m - 1, t)
            };

            l + r
        }
    }

    fn order(&self) -> usize {
        self.knots.len() - self.points.len()
    }

    pub fn degree(&self) -> usize {
        self.order() - 1
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, time::Instant};

    use primitives::{rat, ParamD2, Point3D, Point4d};

    use crate::Curve;

    #[test]
    /*
    pub fn line() {
        let curve = Curve::new(
            vec![
                Point3D::new_ints(0, 0, 0).homogenize_int(1),
                Point3D::new_ints(1, 1, 0).homogenize_int(1),
            ],
            vec![0.into(), 0.into(), 2.into(), 2.into()],
        );

        println!("CURVE {:#?}", curve);

        let num_pts = 10;
        for i in 0..=num_pts {
            let t = rat(i, num_pts) * curve.max_knot();
            let p4d = curve.eval_s(&t);
            println!("p4d {}", p4d);
            let p3d = HPoint::from(p4d.clone()).project();

            println!("t @ {} = {} -> {}", t, p4d, p3d);
        }
    }
    */
    #[test]
    pub fn arc() {
        let curve = Curve::new(
            vec![
                Point4d::new_ints(2, 0, -2, 0),
                Point4d::new_ints(1, 1, -1, 0),
                Point4d::new_ints(1, 1, 1, 0),
                Point4d::new_ints(2, 0, 2, 0),
                Point4d::new_ints(1, -1, 1, 0),
                Point4d::new_ints(1, -1, -1, 0),
                Point4d::new_ints(2, 0, -2, 0),
                /*
                Point3D::new_ints(0, -2, 0).homogenize_int(2),
                Point3D::new_ints(1, -1, 0).homogenize_int(1),
                Point3D::new_ints(1, 1, 0).homogenize_int(1),
                Point3D::new_ints(0, 2, 0).homogenize_int(2),
                Point3D::new_ints(-1, 1, 0).homogenize_int(1),
                Point3D::new_ints(-1, -1, 0).homogenize_int(1),
                Point3D::new_ints(0, -2, 0).homogenize_int(2),
                */
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

        println!("CURVE {:#?}", curve);

        let num_pts = 12;

        let start = Instant::now();

        for i in 0..=num_pts {
            let t = rat(i, num_pts) * curve.max_knot();
            let p4d = curve.eval_s(&t);

            println!("t @ {} = {} ", t, p4d);
            //let p3d = HPoint::from(p4d.clone()).project();
            //println!("t @ {} = {} -> {}", t, p4d, p3d);
        }

        let end = Instant::now();

        println!("{}us", (end - start).as_micros());
    }
}
