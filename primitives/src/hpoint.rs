use crate::{gcd, point3d::Point3D, rat, rational::Rat, Int, Point4d};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HPoint {
    pub(crate) w: Int,
    pub(crate) x: Int,
    pub(crate) y: Int,
    pub(crate) z: Int,
}
impl HPoint {
    pub fn from_rats(w: Rat, x: Rat, y: Rat, z: Rat) -> Self {
        Self::new(
            w.num() * x.den() * y.den() * z.den(),
            x.num() * w.den() * y.den() * z.den(),
            y.num() * w.den() * x.den() * z.den(),
            z.num() * w.den() * x.den() * y.den(),
        )
    }

    pub fn new(w: Int, x: Int, y: Int, z: Int) -> Self {
        if w == 0 && x == 0 && y == 0 && z == 0 {
            Self::zero()
        } else {
            let gcd = gcd(w, gcd(x, gcd(y, z)));

            Self {
                w: w / gcd,
                x: x / gcd,
                y: y / gcd,
                z: z / gcd,
            }
        }
    }

    pub fn zero() -> Self {
        Self {
            w: 0.into(),
            x: 0.into(),
            y: 0.into(),
            z: 0.into(),
        }
    }

    pub fn project(&self) -> Point3D {
        Point3D::new(
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
impl From<Point4d> for HPoint {
    fn from(value: Point4d) -> Self {
        Self::from_rats(value.w, value.x, value.y, value.z)
    }
}
