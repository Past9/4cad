use crate::{Aabb, Point};

pub struct Segment {
    a: Point,
    b: Point,
}
impl Segment {
    pub fn new(start: Point, end: Point) -> Self {
        Self { a: start, b: end }
    }

    pub fn eval(&self, u: f64) -> Point {
        self.a + u * (self.b - self.a)
    }

    pub fn aabb(&self) -> crate::Aabb {
        Aabb {
            min: Point::new(self.a.x.min(self.b.x), self.a.y.min(self.b.y)),
            max: Point::new(self.a.x.max(self.b.x), self.a.y.max(self.b.y)),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn param() {
        let seg = Segment::new(Point::new(1.0, 1.0), Point::new(0.0, 3.0));
        let samples = 100;
        for u in 0..=samples {
            let u = u as f64 / samples as f64;
            let pt = seg.eval(u);
            println!("{:?}", pt);
        }
    }

    #[test]
    fn intersect() {
        let s1 = Segment::new(Point::new(2.0, 1.0), Point::new(5.0, 7.0));
        let s2 = Segment::new(Point::new(2.0, 4.0), Point::new(6.0, 0.0));

        let tx = (s2.a.x - s1.a.x) / (s1.b.x - s1.a.x - s2.b.x + s2.a.x);
        let ty = (s2.a.y - s1.a.y) / (s1.b.y - s1.a.y - s2.b.y + s2.a.y);

        println!("tx = {}", tx);
        println!("ty = {}", ty);
    }
}
