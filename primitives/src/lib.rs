//mod hpoint;
mod paramd2;
mod point3d;
mod point4d;
mod rational;

//pub use hpoint::*;
pub use paramd2::*;
pub use point3d::*;
pub use point4d::*;
pub use rational::*;

pub type Int = i64;
pub type UInt = u128;

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
    // use crate::{rat, EPoint, HPoint};

    /*
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

    #[test]
    pub fn mul_int_rat() {
        let int = 3;
        let rational = rat(1, 5);
        assert_eq!(rat(3, 5), int * rational);
    }

    #[test]
    pub fn mul_hpoint_rat() {
        let hpoint = HPoint::new(1, 2, 3, 4);
        let rat = rat(1, 5);
        println!("{}", hpoint * rat);
    }
    */
}
