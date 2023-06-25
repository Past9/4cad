use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};

use splines::{Curve, EPoint, HPoint, Pt4};
use std::time::Instant;
use viewer::run_viewer;

fn main() {
    // These are the weighted coordinates of points on
    // the start, middle, and end of a quarter circle.
    // Fitting them should generate a quarter-circle arc.
    let curve = Curve::interpolate(
        vec![
            Pt4::new(1.0, 0.0, 0.0, 1.0),
            Pt4::new(
                0.6035533905932737,
                0.6035533905932737,
                0.0,
                0.8535533905932737,
            ),
            Pt4::new(0.0, 1.0, 0.0, 1.0),
        ],
        2,
    );

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
}
