use primitives::{rat, ParamD2, Point4d};
use render::{
    model::{model::Model, Point},
    Vec3,
};
use splines::Curve;
use std::time::Instant;

fn main() {
    let curve = Curve::new(
        vec![
            Point4d::new_ints(2, 0, -2, 0),
            Point4d::new_ints(1, 1, -1, 0),
            Point4d::new_ints(1, 1, 1, 0),
            Point4d::new_ints(2, 0, 2, 0),
            Point4d::new_ints(1, -1, 1, 0),
            Point4d::new_ints(1, -1, -1, 0),
            Point4d::new_ints(2, 0, -2, 0),
        ],
        vec![
            rat(0, 1),
            rat(0, 1),
            rat(0, 1),
            rat(1, 4),
            rat(1, 2),
            rat(1, 2),
            rat(3, 4),
            rat(1, 1),
            rat(1, 1),
            rat(1, 1),
        ],
    );

    let mut model = Model {
        triangles: vec![],
        points: vec![],
        lines: vec![],
    };

    let num_pts = 10;
    let start = Instant::now();
    for i in 0..=num_pts {
        let t = rat(i, num_pts);
        let no = ParamD2::from(t.clone());
        let p4d = curve.eval_i(&t);
        let p3d = p4d.project();

        model.points.push(Point {
            vertex: p3d.into(),
            expand: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        });
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    render::render_model(model, 10.0).unwrap();
}
