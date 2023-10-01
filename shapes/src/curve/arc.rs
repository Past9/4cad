use primitives::Angle;

use crate::Point;

pub struct Arc {
    origin: Point,
    radius: f64,
    start_angle: Angle,
    end_angle: Angle,
}
impl Arc {
    pub fn new(origin: Point, radius: f64, start_angle: Angle, end_angle: Angle) -> Self {
        Self {
            origin,
            radius,
            start_angle,
            end_angle,
        }
    }

    pub fn eval(&self, u: f64) -> Point {
        let sweep_angle = self.end_angle - self.start_angle;
        let angle = self.start_angle + u * sweep_angle;
        Point::new(
            self.origin.x + self.radius * angle.cos(),
            self.origin.y + self.radius * angle.sin(),
        )
    }

    pub fn aabb(&self) -> crate::Aabb {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param() {
        let arc = Arc::new(
            Point::new(2.0, 2.0),
            1.0,
            Angle::deg(0.0),
            Angle::deg(360.0),
        );

        let samples = 100;
        for u in 0..=samples {
            let u = u as f64 / samples as f64;
            let pt = arc.eval(u);
            println!("{:?}", pt);
        }
    }
}
