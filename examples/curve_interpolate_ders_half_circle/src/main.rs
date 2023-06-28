use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, EVec, HVec, Vec3, Vec4};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};

use splines::Curve;
use std::time::Instant;
use viewer::run_viewer;

fn main() {
    let ref_circle = Curve::arc(Angle::deg(180.0));
    println!("ref_circle {:#?}", ref_circle);

    // These are the weighted coordinates of points on
    // the start, middle, and end of a quarter circle.
    // Fitting them should generate a quarter-circle arc.
    let curve = Curve::interpolate_ders(
        vec![
            /*
            Pt4::new(1.0, 0.0, 0.0, 1.0),
            Pt4::new(
                0.6035533905932737,
                0.6035533905932737,
                0.0,
                0.8535533905932737,
            ),
            Pt4::new(0.0, 1.0, 0.0, 1.0),
            */
            (Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(0.0, 1.0, 0.0, 1.0)),
            (
                Vec4::new(
                    0.6035533905932737,
                    0.6035533905932737,
                    0.0,
                    0.8535533905932737,
                ),
                Vec3::new(-1.0, 1.0, 0.0).normalize().to_hpoint(1.0),
            ),
            (
                Vec4::new(0.00000000000000006123233995736766, 1.0, 0.0, 1.0),
                Vec4::new(-1.0, 0.0, 0.0, 1.0),
            ),
            (
                Vec4::new(
                    -0.6035533905932737,
                    0.6035533905932737,
                    0.0,
                    0.8535533905932737,
                ),
                Vec3::new(-1.0, -1.0, 0.0).normalize().to_hpoint(1.0),
            ),
            (
                Vec4::new(-1.0, 0.00000000000000012246467991473532, 0.0, 1.0),
                Vec4::new(0.0, -1.0, 0.0, 1.0),
            ),
        ],
        2,
    );

    let mut points = Vec::new();

    let num_pts = 270;
    let start = Instant::now();
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = curve.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    let ref_pts = 8;
    for i in 0..=ref_pts {
        let t = i as f64 / ref_pts as f64;
        let p4d = ref_circle.eval(t);
        println!("Pt4::new({}, {}, {}, {}),", p4d.x, p4d.y, p4d.z, p4d.w);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::RED));
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
}
