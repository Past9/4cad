use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::Angle;
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};

use splines::{Curve, EPoint, HPoint};
use std::time::Instant;
use viewer::run_viewer;

fn main() {
    let curve = Curve::arc(Angle::deg(360.0)).elevate_degree(1);

    let mut points = Vec::new();

    let num_pts = 100;
    let start = Instant::now();
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = curve.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(
            0.into(),
            p3d.as_f32(),
            Vector3::zero(),
            Rgba::WHITE,
        ));
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

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
    */
}
