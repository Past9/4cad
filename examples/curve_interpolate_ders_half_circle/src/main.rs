use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};

use splines::{Curve, KnotVec};
use std::time::Instant;
use viewer::run_viewer;

fn main() {
    let ref_circle =
        Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)));

    /*
    let ref_circle = Curve::unweighted(
        vec![
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(0.0, -1.0, 0.0, 1.0),
            Vec4::new(-1.0, 0.0, 0.0, 1.0),
        ],
        KnotVec::uniform(3, 2),
    );
    */
    println!("ref_circle {:#?}", ref_circle);

    let mut points_ders: Vec<(Vec4, Vec4)> = vec![];
    let num_pts = 5;
    for i in 0..num_pts {
        let t = i as f64 / (num_pts - 1) as f64;
        let point_der = ref_circle.eval_derivatives(t, 1);
        points_ders.push((
            point_der[0], //.project().to_hpoint(1.0),
            point_der[1], //.project().to_hpoint(1.0),
        ));
    }

    println!("ref_circle points_ders {:#?}", points_ders);

    //panic!();

    // These are the weighted coordinates of points on
    // the start, middle, and end of a quarter circle.
    // Fitting them should generate a quarter-circle arc.
    let rt = 2f64.sqrt() / 2.0;
    let curve = Curve::interpolate_ders(
        /*
        vec![
            /*
            (Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(0.0, 1.0, 0.0, 1.0)),
            (
                Vec4::new(
                    0.6035533905932737,
                    0.6035533905932737,
                    0.0,
                    0.8535533905932737,
                ),
                Vec4::new(-rt, rt, 0.0, 1.0),
            ),
            (
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(-1.0, 0.0, 0.0, 1.0),
            ),
            */
            (Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(0.0, 1.0, 0.0, 1.0)),
            (Vec4::new(rt, rt, 0.0, 1.0), Vec4::new(-rt, rt, 0.0, 1.0)),
            (
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(-1.0, 0.0, 0.0, 1.0),
            ),
            (Vec4::new(-rt, rt, 0.0, 1.0), Vec4::new(-rt, -rt, 0.0, 1.0)),
            (
                Vec4::new(-1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, -1.0, 0.0, 1.0),
            ),
        ],
        */
        points_ders,
        2,
    );

    let mut points = Vec::new();

    let eval_pts = 97;
    let start = Instant::now();
    for i in 0..=eval_pts {
        let t = i as f64 / eval_pts as f64;
        let p4d = curve.eval_pos(t);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::WHITE));
    }
    let end = Instant::now();
    println!("{}us", (end - start).as_micros());

    let num_eval_pts = 170;
    for i in 0..=(num_eval_pts - 1) {
        let t = i as f64 / (num_eval_pts - 1) as f64;
        let p4d = ref_circle.eval_pos(t);
        //println!("Pt4::new({}, {}, {}, {}),", p4d.x, p4d.y, p4d.z, p4d.w);
        let p3d = p4d.project();

        points.push(ModelPoint::new(0.into(), p3d, Vector3::zero(), Rgba::RED));
    }

    let ctrl_pts = curve
        .clone()
        .take_weighted()
        .into_iter()
        .map(|pt| pt.project())
        .collect::<Vec<_>>();
    println!("ctrl_pts.len() {}", ctrl_pts.len());
    for i in 0..ctrl_pts.len() {
        points.push(ModelPoint::new(
            0.into(),
            ctrl_pts[i].into(),
            Vector3::zero(),
            Rgba::MAGENTA,
        ));
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
