use primitives::{EPoint, HPoint, Param, Rat};

pub struct Curve {
    points: Vec<EPoint>,
    weights: Vec<Rat>,
    knots: Vec<Rat>,
}
impl Curve {
    pub fn to_type_s(&self) -> CurveS {
        CurveS {
            points: self
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| p.clone().homogenize(self.weights[i].clone()))
                .collect(),
            knots: self.knots.clone(),
        }
    }
}

pub struct CurveS {
    points: Vec<HPoint>,
    knots: Vec<Rat>,
}
impl CurveS {
    pub fn eval(&self, t: &Rat) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }

    pub fn basis(&self, j: usize, m: usize, t: &Rat) -> Rat {
        if m == 1 {
            let ti = &self.knots[j];
            let ti1 = &self.knots[j + 1];

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

            ((t - tj) / (tjmsub1 - tj)) * self.basis(j, m - 1, t)
                + ((tjm - t) / (tjm - tj1)) * self.basis(j + 1, m - 1, t)
        }
    }

    pub fn degree(&self) -> usize {
        self.knots.len() - self.points.len() - 1
    }
}

pub struct CurveD1 {
    points: Vec<HPoint>,
    knots: Vec<Rat>,
}
impl CurveD1 {
    pub fn eval(&self, t: &Rat) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }

    pub fn basis(&self, j: usize, m: usize, t: &Rat) -> Rat {
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

            ((s - h * tj) / (tjmsub1 - tj)) * self.basis(j, m - 1, t)
                + ((h * tjm - s) / (tjm - tj1)) * self.basis(j + 1, m - 1, t)
        }
    }

    pub fn degree(&self) -> usize {
        self.knots.len() - self.points.len() - 1
    }
}

pub struct CurveD2 {
    points: Vec<HPoint>,
    knots: Vec<Rat>,
}
impl CurveD2 {
    pub fn eval(&self, t: &Param) -> HPoint {
        let m = self.degree();
        (0..self.points.len())
            .map(|j| self.basis(j, m, &t) * &self.points[j])
            .sum::<HPoint>()
    }

    pub fn basis(&self, j: usize, m: usize, t: &Param) -> Rat {
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

            ((o - (n + o) * tj) / (tjmsub1 - tj)) * self.basis(j, m - 1, t)
                + (((n + o) * tjm - o) / (tjm - tj1)) * self.basis(j + 1, m - 1, t)
        }
    }

    pub fn degree(&self) -> usize {
        self.knots.len() - self.points.len() - 1
    }
}
