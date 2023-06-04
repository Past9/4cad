use std::ops::Sub;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};

use crate::{gcd, Int};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Rat {
    num: Int,
    den: Int,
}
impl Rat {
    pub fn new(num: Int, den: Int) -> Self {
        if den == 0 {
            return Self::zero();
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

    pub fn one() -> Self {
        Self { num: 1, den: 1 }
    }

    pub fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    pub fn is_zero(&self) -> bool {
        self.num == 0 && self.den != 0
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
