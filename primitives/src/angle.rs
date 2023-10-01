use std::f64::consts::PI;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};

#[derive(Debug, Clone, Copy)]
pub struct Angle(pub f64);
impl Angle {
    pub fn rad(rad: f64) -> Self {
        Self(rad)
    }

    pub fn deg(deg: f64) -> Self {
        Self(deg * PI / 180.0)
    }

    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    pub fn tan(self) -> f64 {
        self.0.tan()
    }
}

impl_op_ex!(+|a: Angle, b: Angle| -> Angle { Angle(a.0 + b.0) });
impl_op_ex!(+=|a: &mut Angle, b: Angle| { a.0 += b.0 });

impl_op_ex!(-|a: Angle, b: Angle| -> Angle { Angle(a.0 - b.0) });
impl_op_ex!(-=|a: &mut Angle, b: Angle| { a.0 -= b.0 });

impl_op_ex_commutative!(*|a: Angle, b: f64| -> Angle { Angle(a.0 * b) });
impl_op_ex!(/|a: Angle, b: f64| -> Angle { Angle(a.0 / b) });
impl_op_ex!(/|a: Angle, b: Angle| -> f64 { a.0 / b.0 });
