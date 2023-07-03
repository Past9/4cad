use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};
use splines::{Curve, Surface};
use viewer::run_viewer;

fn main() {
    let profile = Curve::arc(Angle::deg(360.0))
        .transform(&(Mat4::from_angle_z(Deg(-90.0)) * Mat4::from_angle_y(Deg(90.0))));
    let trajectory = Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_scale(2.0));

    let num_sections = 0;
    let (_, _, sections) = Surface::generate_sweep_section_curves(&profile, &trajectory, 1.0);
    let surface = Surface::sweep_curve(&profile, &trajectory, 1.0);

    let inner_edge = Curve::arc(Angle::deg(360.0));
    let outer_edge = Curve::arc(Angle::deg(360.0)).transform(&Mat4::from_scale(3.0));

    let mut points = Vec::new();

    let num_pts = 200;
    let start = Instant::now();
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

    // Profile curve
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = profile.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::RED));
    }

    // Trajectory curve
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = trajectory.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::GREEN));
    }

    // Inner edge curve
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = inner_edge.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::BLUE));
    }

    // Outer edge curve
    for i in 0..=num_pts {
        let t = i as f64 / num_pts as f64;
        let p4d = outer_edge.eval(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::BLUE));
    }

    // Section curves
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

    // Section curve control points
    for section in sections.iter() {
        for i in 0..section.num_pts() {
            points.push(ModelPoint::new(
                0.into(),
                section.clone().take_unweighted()[i].truncate(),
                Vector3::zero(),
                Rgba::ORANGE,
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
