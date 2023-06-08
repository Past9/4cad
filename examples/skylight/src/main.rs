use primitives::{rat, ParamD2, Point4d};
/*
use render::{
    model::{model::Model, Point},
    Vec3,
};
*/
use splines::{Curve, Surface};
use std::time::Instant;

fn main() {
    /*
    let surface = Surface::new(
        vec![
            vec![
                Point4d::new_ints(1, -1, 0, -1),
                Point4d::new_ints(1, 0, 0, -1),
                Point4d::new_ints(1, 1, 0, -1),
            ],
            vec![
                Point4d::new_ints(1, -1, 0, 0),
                Point4d::new_ints(2, 0, -3, 0),
                Point4d::new_ints(1, 1, 0, 0),
            ],
            vec![
                Point4d::new_ints(1, -1, 0, 1),
                Point4d::new_ints(1, 0, 0, 1),
                Point4d::new_ints(1, 1, 0, 1),
            ],
        ],
        vec![
            rat(0, 1),
            rat(0, 1),
            rat(0, 1),
            rat(1, 1),
            rat(1, 1),
            rat(1, 1),
        ],
        vec![
            rat(0, 1),
            rat(0, 1),
            rat(0, 1),
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
        for j in 0..=num_pts {
            let u = rat(i, num_pts);
            let v = rat(j, num_pts);
            let no_u = ParamD2::from(rat(i, num_pts));
            let no_v = ParamD2::from(rat(j, num_pts));
            let p4d = surface.eval_i(&u, &v);
            let p3d = p4d.project();

            println!("({}, {}) -> {}", u, v, p3d);

            model.points.push(Point {
                vertex: p3d.into(),
                expand: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            });
        }
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    render::render_model(model, 10.0).unwrap();
    */
}
