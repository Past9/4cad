use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};
use splines::{Curve, KnotVec, Surface};
use viewer::run_viewer;

fn main() {
    let curves = vec![
        Curve::arc(Angle::deg(180.0)),
        Curve::create_unweighted(
            vec![
                Vec4::new(1.0, 0.0, 1.0, 1.0),
                Vec4::new(0.5, -0.5, 1.0, 1.0),
                Vec4::new(0.0, -0.5, 1.0, 1.0),
                Vec4::new(-0.5, -0.5, 1.0, 1.0),
                Vec4::new(-1.0, 0.0, 1.0, 1.0),
            ],
            KnotVec::uniform(5, 2),
        ),
        Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_translation(Vec3::new(0.0, 0.0, 2.0))),
        Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_translation(Vec3::new(0.0, 1.0, 3.0))),
        Curve::arc(Angle::deg(180.0))
            .transform(&Mat4::from_angle_x(Deg(180.0)))
            .transform(&Mat4::from_translation(Vec3::new(0.0, 2.0, 4.0))),
    ];
    let surface = Surface::skin_curves(&curves, 3);

    let mut points = Vec::new();

    let start = Instant::now();
    let num_pts = 200;
    for i in 0..=num_pts {
        for j in 0..=num_pts {
            let u = i as f64 / num_pts as f64;
            let v = j as f64 / num_pts as f64;
            let p4d = surface.eval_pos(u, v);
            let p3d = p4d.project();

            points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
        }
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    for curve in curves.iter() {
        for i in 0..=num_pts {
            let t = i as f64 / num_pts as f64;
            let p4d = curve.eval_pos(t);
            let p3d = p4d.project();

            points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::RED));
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
}
