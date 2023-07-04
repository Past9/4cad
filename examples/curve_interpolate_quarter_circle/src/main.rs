use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};

use splines::{Curve, Surface};
use std::time::Instant;
use viewer::run_viewer;

fn main() {
    let ref_circle = Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_scale(2.0));

    let mut points = vec![];
    let num_pts = 5;
    for i in 0..num_pts {
        let t = i as f64 / (num_pts - 1) as f64;
        let point = ref_circle.eval(t);
        points.push(point);
    }

    println!("interpolate points {:#?}", points);

    // These are the weighted coordinates of points on
    // the start, middle, and end of a quarter circle.
    // Fitting them should generate a quarter-circle arc.
    let curve = Curve::interpolate(points, 2, None, Some(ref_circle.knots().clone()));

    let surface = Surface::rule_curve(
        curve
            .clone()
            .transform(&Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0))),
        Vec3::new(0.0, 0.0, 2.0),
    );

    println!("curve {:#?}", curve);

    println!("surface {:#?}", surface);

    let mut points = Vec::new();

    let num_pts = 180;
    let start = Instant::now();
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = curve.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    for i in 0..=num_pts {
        for j in 0..=num_pts {
            let u = i as f64 / num_pts as f64;
            let v = j as f64 / num_pts as f64;
            let p4d = surface.eval(u, v);
            let p3d = p4d.project();

            points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
        }
    }

    let ref_pts = 400;
    for i in 0..=ref_pts {
        let t = i as f64 / ref_pts as f64;
        let p4d = ref_circle.eval(t);
        //println!("Pt4::new({}, {}, {}, {}),", p4d.x, p4d.y, p4d.z, p4d.w);
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
