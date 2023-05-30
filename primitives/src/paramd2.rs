use crate::{gcd, rational::Rat, Int};

pub struct ParamD2 {
    n: Int,
    o: Int,
}
impl ParamD2 {
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
impl From<Rat> for ParamD2 {
    fn from(value: Rat) -> Self {
        ParamD2::hs(value.num(), value.den())
    }
}
impl From<u8> for ParamD2 {
    fn from(value: u8) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<i8> for ParamD2 {
    fn from(value: i8) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<u16> for ParamD2 {
    fn from(value: u16) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<i16> for ParamD2 {
    fn from(value: i16) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<u32> for ParamD2 {
    fn from(value: u32) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<i32> for ParamD2 {
    fn from(value: i32) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
impl From<u64> for ParamD2 {
    fn from(value: u64) -> Self {
        ParamD2::hs(value as Int, 1)
    }
}
