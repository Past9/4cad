use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::Point4d;
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};
/*
use render::{
    model::{model::Model, Point},
    Vec3,
};
*/
use splines::{Curve, Surface};
use std::time::Instant;
use viewer::run_viewer;

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
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    */

    let w = 2f64.sqrt() / 2.0;
    let surface = Surface::new(
        vec![
            vec![
                Point4d::new(1.0, 1.0, 0.0, 0.0),
                Point4d::new(w, 1.0, 1.0, 0.0),
                Point4d::new(1.0, 0.0, 1.0, 0.0),
                Point4d::new(w, -1.0, 1.0, 0.0),
                Point4d::new(1.0, -1.0, 0.0, 0.0),
                Point4d::new(w, -1.0, -1.0, 0.0),
                Point4d::new(1.0, 0.0, -1.0, 0.0),
                Point4d::new(w, 1.0, -1.0, 0.0),
                Point4d::new(1.0, 1.0, 0.0, 0.0),
            ],
            vec![
                Point4d::new(1.0, 1.0, 0.0, 3.0),
                Point4d::new(w, 1.0, 1.0, 3.0),
                Point4d::new(1.0, 0.0, 1.0, 3.0),
                Point4d::new(w, -1.0, 1.0, 3.0),
                Point4d::new(1.0, -1.0, 0.0, 3.0),
                Point4d::new(w, -1.0, -1.0, 3.0),
                Point4d::new(1.0, 0.0, -1.0, 3.0),
                Point4d::new(w, 1.0, -1.0, 3.0),
                Point4d::new(1.0, 1.0, 0.0, 3.0),
            ],
        ],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![
            0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
        ],
    );

    let mut points = Vec::new();

    let num_pts = 50;
    for i in 0..=num_pts {
        for j in 0..=num_pts {
            let u = i as f64 / num_pts as f64;
            let v = j as f64 / num_pts as f64;
            let p4d = surface.eval(u, v);
            let p3d = p4d.project();

            println!("({}, {}) -> {}", u, v, p3d);

            points.push(ModelPoint::new(
                0.into(),
                p3d.into(),
                Vector3::zero(),
                Rgba::WHITE,
            ));
        }
    }

    let model = Model::empty().points(points);

    let mut geometry = Geometry::new();
    geometry.insert_model(model);

    let mut sb = SceneBuilder::empty();
    sb.background(rgba(0.05, 0.1, 0.15, 1.0))
        .camera(Camera::create_perspective(
            [0, 0],
            point3(0.0, 0.0, -3.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, -1.0, 0.0).normalize(),
            Deg(70.0).into(),
            0.01,
            5.0,
        ))
        .geometry(geometry);

    run_viewer(sb.build());
    /*
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
