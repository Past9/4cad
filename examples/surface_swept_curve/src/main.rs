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
    let profile = Curve::arc(Angle::deg(360.0))
        .transform(&(Mat4::from_angle_z(Deg(-90.0)) * Mat4::from_angle_y(Deg(90.0))))
        .transform(&Mat4::from_scale(0.5));

    let trajectory = Curve::unweighted(
        vec![
            Vec4::new(-2.0, 0.0, 0.0, 1.0),
            Vec4::new(-1.0, -1.0, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 1.0, 0.0, 1.0),
            Vec4::new(2.0, 0.0, 0.0, 1.0),
        ],
        KnotVec::uniform(5, 2),
    );
    //.transform(&Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)));
    //let trajectory = Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_scale(2.0));

    let num_sections = 30;
    let (_, _, sections) =
        Surface::generate_sweep_section_curves(&profile, &trajectory, num_sections);
    let surface = Surface::sweep_curve(&profile, &trajectory, num_sections);

    let edge = Curve::arc(Angle::deg(360.0)).transform(&Mat4::from_scale(3.0));

    let mut points = Vec::new();

    let start = Instant::now();
    let num_pts = 100;
    for i in 0..=num_pts {
        for j in 0..=num_pts {
            let u = i as f64 / num_pts as f64;
            let v = j as f64 / num_pts as f64;
            let p4d = surface.eval(u, v);
            let p3d = p4d.project();

            points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
        }
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = profile.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::RED));
    }

    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = trajectory.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::GREEN));
    }

    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = edge.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::BLUE));
    }

    for section in sections.iter() {
        let num_pts = num_pts * 10;
        for i in 0..=num_pts {
            let t = i as f64 / num_pts as f64;
            let p4d = section.eval(t);
            let p3d = p4d.project();

            points.push(ModelPoint::new(
                0.into(),
                p3d,
                Vector3::zero(),
                Rgba::YELLOW,
            ));
        }
    }

    for section in sections.iter() {
        for i in 0..section.num_pts() {
            points.push(ModelPoint::new(
                0.into(),
                section.clone().take_unweighted()[i].truncate(),
                Vector3::zero(),
                Rgba::MAGENTA,
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
}
