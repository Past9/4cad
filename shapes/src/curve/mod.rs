mod arc;
mod segment;

pub use arc::*;
use primitives::Angle;
pub use segment::*;

use crate::Point;

pub struct Aabb {
    min: Point,
    max: Point,
}

pub enum Curve {
    Segment(Segment),
    Arc(Arc),
}
impl Curve {
    pub fn arc(origin: Point, radius: f64, start_angle: Angle, end_angle: Angle) -> Self {
        Self::Arc(Arc::new(origin, radius, start_angle, end_angle))
    }

    pub fn segment(start: Point, end: Point) -> Self {
        Self::Segment(Segment::new(start, end))
    }

    pub fn eval(&self, u: f64) -> Point {
        match self {
            Curve::Segment(segment) => segment.eval(u),
            Curve::Arc(arc) => arc.eval(u),
        }
    }

    pub fn aabb(&self) -> Aabb {
        todo!()
    }

    pub fn intersect(&self, other: &Curve) -> Vec<CCIntersection> {
        match (self, other) {
            (Curve::Segment(segment), Curve::Arc(arc))
            | (Curve::Arc(arc), Curve::Segment(segment)) => todo!(),
            (Curve::Arc(_), Curve::Arc(_)) => todo!(),
            (Curve::Segment(_), Curve::Segment(_)) => todo!(),
        }
    }
}

/*
pub trait CurveImpl {
    fn eval(&self, u: f64) -> Point;
    fn aabb(&self) -> Aabb;
    fn intersect(&self, other: Curve) -> Vec<CCIntersection>;
}
 */

pub enum CCIntersection {
    Curve(Curve),
    Point(Point),
}
