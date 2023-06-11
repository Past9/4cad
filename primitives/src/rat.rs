use std::ops::{Add, Div, Mul, Sub};

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use cgmath::{One, Zero};

use crate::{gcd, Int};

const CONT_FRAC_ITER: usize = 4;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Angle(Rat);
impl Angle {
    pub fn deg(deg: Rat) -> Self {
        Self(deg * Rat::PI1_180)
    }

    pub fn rad(rad: Rat) -> Self {
        Self(rad)
    }

    pub fn new_deg(num: Int, den: Int) -> Self {
        Self::deg(Rat::new(num, den))
    }

    pub fn new_rad(num: Int, den: Int) -> Self {
        Self::rad(Rat::new(num, den))
    }

    fn sin(&self) -> Rat {
        let tan = (self / Rat::new(2, 1)).tan();

        Rat::new(2 * tan.den * tan.num, tan.den.pow(2) + tan.num.pow(2))
    }

    fn cos(&self) -> Rat {
        let tan = (self / Rat::new(2, 1)).tan();

        let tan2 = tan.powi(2);
        Rat::new(tan2.den - tan2.num, tan2.den + tan2.num)
    }

    fn tan(&self) -> Rat {
        let rads = &self.0;
        rads / Self::rec_tan(*rads, rads.powi(2), 0, CONT_FRAC_ITER)
    }

    fn rec_tan(rads: Rat, rads2: Rat, iter: usize, max_iter: usize) -> Rat {
        if iter > max_iter {
            return Rat::ONE;
        }

        let n = Rat {
            num: (2 * iter + 1) as Int,
            den: 1,
        };

        n - rads2 / Self::rec_tan(rads, rads2, iter + 1, max_iter)
    }
}
impl_op_ex!(-|a: &Angle| -> Angle { Angle(-a.0) });
impl_op_ex!(+ |a: &Angle, b: &Angle| -> Angle { Angle(a.0 + b.0) });
impl_op_ex!(-|a: &Angle, b: &Angle| -> Angle { Angle(a.0 - b.0) });

impl_op_ex_commutative!(*|a: &Angle, b: &Rat| -> Angle { Angle(a.0 * b) });
impl_op_ex!(/|a: &Angle, b: &Rat| -> Angle { Angle(a.0 / b) });

/*
pub trait Angle {
    fn rads(&self) -> Rat;

    fn sin(&self) -> Rat {
        let tan = (self / Rat::new(2, 1)).tan();

        Rat::new(2 * tan.den * tan.num, tan.den.pow(2) + tan.num.pow(2))
    }

    /*
    fn cos(&self) -> Rat {
        let tan = self.tan();

        todo!()
    }
    */

    fn tan(&self) -> Rat {
        let rads = &self.rads();
        rads / Self::rec_tan(*rads, rads.powi(2), 0, CONT_FRAC_ITER)
    }

    fn rec_tan(rads: Rat, rads2: Rat, iter: usize, max_iter: usize) -> Rat {
        if iter > max_iter {
            return Rat::ONE;
        }

        let n = Rat {
            num: (2 * iter + 1) as i64,
            den: 1,
        };

        n - rads2 / Self::rec_tan(rads, rads2, iter + 1, max_iter)
    }
}
*/

/*
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Rad(Rat);
impl Rad {
    pub fn new(num: Int, den: Int) -> Self {
        Self(Rat::new(num, den))
    }
}
impl Angle for Rad {
    fn rads(&self) -> Rat {
        self.0
    }
}
impl Mul<Rat> for Rad {
    type Output = Rad;

    fn mul(self, rhs: Rat) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Mul<Rad> for Rat {
    type Output = Rad;

    fn mul(self, rhs: Rad) -> Self::Output {
        Rad(self * rhs.0)
    }
}
impl Div<Rat> for Rad {
    type Output = Rad;

    fn div(self, rhs: Rat) -> Self::Output {
        Self(self.0 / rhs)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Deg(Rat);
impl Deg {
    pub fn new(num: Int, den: Int) -> Self {
        Self(Rat::new(num, den))
    }
}
impl Angle for Deg {
    fn rads(&self) -> Rat {
        self.0 * Rat::PI1_180
    }
}
impl Mul<Rat> for Deg {
    type Output = Deg;

    fn mul(self, rhs: Rat) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Mul<Deg> for Rat {
    type Output = Deg;

    fn mul(self, rhs: Deg) -> Self::Output {
        Deg(self * rhs.0)
    }
}
impl Div<Rat> for Deg {
    type Output = Deg;

    fn div(self, rhs: Rat) -> Self::Output {
        Self(self.0 / rhs)
    }
}
*/

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Rat {
    num: Int,
    den: Int,
}
impl Rat {
    pub fn new(num: Int, den: Int) -> Self {
        if den == 0 {
            //return Self::zero();
            panic!("Denominator of {}/{} is zero", num, den);
        }

        let gcd = gcd(num, den);

        let num = num / gcd;
        let den = den / gcd;

        if den.is_negative() {
            Self {
                num: -num,
                den: -den,
            }
        } else {
            Self { num, den }
        }
    }

    pub const EPS: Rat = Rat {
        num: 1,
        den: 10000000,
    };

    pub const ONE: Rat = Rat { num: 1, den: 1 };

    pub const ZERO: Rat = Rat { num: 0, den: 1 };

    pub const PI: Rat = Rat {
        num: 1146408,
        den: 364913,
    };

    pub const PI1_180: Rat = Rat {
        num: 95534,
        den: 5473695,
    };

    pub const PI2: Rat = Rat {
        num: 103338,
        den: 364913,
    };

    pub fn approx_eq(&self, rhs: &Self) -> bool {
        (self - rhs).abs() < Self::EPS
    }

    pub fn abs(self) -> Self {
        Self {
            num: self.num.abs(),
            den: self.den,
        }
    }

    pub fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    pub fn is_zero(&self) -> bool {
        self.num == 0 && self.den != 0
    }

    pub fn is_one(&self) -> bool {
        self.num == self.den
    }

    pub fn expect_int(&self) -> Int {
        if self.den == 1 {
            self.num
        } else {
            panic!("{} is not an integer", self);
        }
    }

    pub fn num(&self) -> Int {
        self.num
    }

    pub fn den(&self) -> Int {
        self.den
    }

    pub fn num_den(&self) -> (Int, Int) {
        (self.num(), self.den())
    }

    pub fn powi(self, exp: Int) -> Self {
        if exp.is_zero() {
            Self::ZERO
        } else if exp.is_one() {
            self
        } else if exp.is_negative() {
            Self {
                num: self.den.pow(exp as u32),
                den: self.num.pow(exp as u32),
            }
        } else {
            Self {
                num: self.num.pow(exp as u32),
                den: self.den.pow(exp as u32),
            }
        }
    }
}
impl std::fmt::Display for Rat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.num, self.den))
    }
}
impl From<Rat> for f32 {
    fn from(value: Rat) -> Self {
        value.num as f32 / value.den as f32
    }
}
impl From<Rat> for f64 {
    fn from(value: Rat) -> Self {
        value.num as f64 / value.den as f64
    }
}
impl PartialOrd for Rat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let l = self.num * other.den;
        let r = other.num * self.den;
        l.partial_cmp(&r)
    }
}
impl From<Int> for Rat {
    fn from(value: Int) -> Self {
        Rat { num: value, den: 1 }
    }
}

impl_op_ex!(-|a: &Rat| -> Rat { Rat::new(-a.num, a.den) });
impl_op_ex!(+ |a: &Rat, b: &Rat| -> Rat { Rat::new(a.num * b.den + b.num * a.den, a.den * b.den) });
impl_op_ex!(-|a: &Rat, b: &Rat| -> Rat { Rat::new(a.num * b.den - b.num * a.den, a.den * b.den) });
impl_op_ex!(*|a: &Rat, b: &Rat| -> Rat { Rat::new(a.num * b.num, a.den * b.den) });
impl_op_ex!(/|a: &Rat, b: &Rat| -> Rat { Rat::new(a.num * b.den, a.den * b.num) });

impl Sub<Int> for Rat {
    type Output = Self;

    fn sub(self, rhs: Int) -> Self::Output {
        Self::new(self.num - rhs * self.den, self.den)
    }
}
impl Sub<Rat> for Int {
    type Output = Rat;

    fn sub(self, rhs: Rat) -> Self::Output {
        Rat::new(self * rhs.den - rhs.num, rhs.den)
    }
}

impl_op_ex_commutative!(+ |a: &Rat, b: &Int| -> Rat { Rat::new(a.num + b * a.den, a.den) });
impl_op_ex_commutative!(*|a: &Rat, b: &Int| -> Rat { Rat::new(a.num * b, a.den) });

pub fn rat(num: Int, den: Int) -> Rat {
    Rat::new(num, den)
}

#[cfg(test)]
mod tests {
    use crate::{Angle, Rat};

    const FLOAT_EPS: f64 = 0.000000001;

    fn test_eq(actual: Rat, expected: f64) {
        let err = f64::from(actual) - expected;
        assert!(
            err.abs() < FLOAT_EPS,
            "Error {} is greater than epsilon {} (actual = {}, actual_float = {}, expected = {})",
            err,
            FLOAT_EPS,
            actual,
            f64::from(actual),
            expected
        );
    }

    #[test]
    fn calcs_tan() {
        test_eq(Angle::new_rad(1, 2).tan(), 0.5f64.tan());
    }

    #[test]
    fn calcs_sin() {
        test_eq(Angle::new_rad(1, 2).sin(), 0.5f64.sin());
    }

    #[test]
    fn calcs_cos() {
        test_eq(Angle::new_rad(1, 2).cos(), 0.5f64.cos());
        //println!("{}", Angle::new_deg(90, 1).cos());
    }
}
