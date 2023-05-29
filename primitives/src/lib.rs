use std::{
    borrow::Borrow,
    iter::Sum,
    ops::{Add, Div, IndexMut, Mul, Neg, Sub},
};

use auto_ops::{impl_op_ex, impl_op_ex_commutative};

pub type Int = i128;
pub type UInt = u128;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Rat {
    num: Int,
    den: Int,
}
impl Rat {
    pub fn new(num: Int, den: Int) -> Self {
        let gcd = gcd(num, den);
        println!("num den gcd {} {} {}", num, den, gcd);
        Self {
            num: num / gcd,
            den: den / gcd,
        }
    }

    pub fn one() -> Self {
        Self { num: 1, den: 1 }
    }

    pub fn zero() -> Self {
        Self { num: 0, den: 1 }
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
}
impl std::fmt::Display for Rat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.num, self.den))
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EPoint {
    x: Rat,
    y: Rat,
    z: Rat,
}
impl EPoint {
    pub fn new(x: Rat, y: Rat, z: Rat) -> Self {
        Self { x, y, z }
    }

    pub fn new_ints(x: Int, y: Int, z: Int) -> Self {
        Self::new(x.into(), y.into(), z.into())
    }

    pub fn homogenize(self, w: Rat) -> HPoint {
        HPoint::from_rats(w, self.x, self.y, self.z)
    }

    pub fn homogenize_int(self, w: Int) -> HPoint {
        self.homogenize(w.into())
    }
}
impl std::fmt::Display for EPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.x, self.y, self.z))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct HPoint {
    w: Int,
    x: Int,
    y: Int,
    z: Int,
}
impl HPoint {
    pub fn from_rats(w: Rat, x: Rat, y: Rat, z: Rat) -> Self {
        Self::new(
            w.num * x.den * y.den * z.den,
            x.num * w.den * y.den * z.den,
            y.num * w.den * x.den * z.den,
            z.num * w.den * x.den * y.den,
        )
    }

    pub fn new(w: Int, x: Int, y: Int, z: Int) -> Self {
        let gcd = gcd(w, gcd(x, gcd(y, z)));

        Self {
            w: w / gcd,
            x: x / gcd,
            y: y / gcd,
            z: z / gcd,
        }
    }

    pub fn zero() -> Self {
        Self {
            w: 1,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    pub fn project(&self) -> EPoint {
        EPoint::new(
            rat(self.x, self.w),
            rat(self.y, self.w),
            rat(self.z, self.w),
        )
    }
}
impl std::fmt::Display for HPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}, {}, {}, {})",
            self.w, self.x, self.y, self.z
        ))
    }
}
impl Sum for HPoint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut point = HPoint::zero();
        for item in iter {
            point = point + item;
        }
        point
    }
}

impl_op_ex!(+ |a: &HPoint, b: &HPoint| -> HPoint {
    HPoint::new(
        a.w + b.w,
        a.x + b.x,
        a.y + b.y,
        a.z + b.z,
    )
});

impl_op_ex_commutative!(*|a: &HPoint, b: &Rat| -> HPoint {
    HPoint::from_rats(a.w * b, a.x * b, a.y * b, a.z * b)
});

pub struct Param {
    n: Int,
    o: Int,
}
impl Param {
    pub fn hs(h: Int, s: Int) -> Self {
        let gcd = gcd(h, s);
        let h = h / gcd;
        let s = s / gcd;

        Self::no(h - s, s)
    }

    pub fn no(n: Int, o: Int) -> Self {
        Self { n, o }
    }

    pub fn n(&self) -> Int {
        self.n
    }

    pub fn o(&self) -> Int {
        self.n
    }
}
impl From<Rat> for Param {
    fn from(value: Rat) -> Self {
        Param::hs(value.num, value.den)
    }
}
impl From<u8> for Param {
    fn from(value: u8) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<i8> for Param {
    fn from(value: i8) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<u16> for Param {
    fn from(value: u16) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<i16> for Param {
    fn from(value: i16) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<u32> for Param {
    fn from(value: u32) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<i32> for Param {
    fn from(value: i32) -> Self {
        Param::hs(value as i128, 1)
    }
}
impl From<u64> for Param {
    fn from(value: u64) -> Self {
        Param::hs(value as i128, 1)
    }
}

pub fn gcd(a: Int, b: Int) -> Int {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }

    a
}

#[cfg(test)]
mod tests {
    use crate::{rat, EPoint, HPoint};

    #[test]
    pub fn homogenizes_epoint() {
        let epoint = EPoint::new(rat(1, 2), rat(1, 3), rat(1, 4));
        assert_eq!(HPoint::new(12, 30, 20, 15), epoint.homogenize(rat(1, 5)));
    }

    #[test]
    pub fn projects_hpoint() {
        let hpoint = HPoint::new(12, 30, 20, 15);
        assert_eq!(
            EPoint::new(rat(5, 2), rat(5, 3), rat(5, 4),),
            hpoint.project()
        );
    }
}
