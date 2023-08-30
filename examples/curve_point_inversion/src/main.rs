use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Zero};
use primitives::{HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    lights::Lights,
    model::{Geometry, Model, ModelEdge, ModelPoint},
    rgb, rgba,
    scene::SceneBuilder,
    Rgb, Rgba,
};
use splines::{Curve, KnotVec};
use tessellate::curve::CurveTessellation;
use viewer::run_viewer;

fn main() {
    let mut geometry = Geometry::new();
    let mut model = Model::empty();
    let resolution = 1000;

    let curve = Curve::create_unweighted(
        vec![
            Vec4::new(-2.0, 0.0, 1.0, 1.0),
            Vec4::new(-1.0, 1.0, 0.0, 2.0),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            Vec4::new(1.0, -1.0, 0.0, 0.5),
            Vec4::new(2.0, 0.0, -1.0, 1.0),
        ],
        KnotVec::from([
            // knots
            0.0, 0.0, 0.0, 0.3, 0.7, 1.0, 1.0, 1.0,
        ]),
    );

    let res = 50;

    let points_on_curve = (0..=res)
        .map(|u| {
            curve
                .transform(&Mat4::from_translation(vec3(0.0, 0.0, 0.0)))
                .eval_pos(u as f64 / res as f64)
                .project()
                + vec3(0.0, 0.0, 0.0)
        })
        .collect::<Vec<_>>();

    let points_not_on_curve = points_on_curve
        .iter()
        .flat_map(|pt| vec![pt + vec3(0.1, 0.1, 0.1)])
        .collect::<Vec<_>>();

    let points = points_on_curve
        .into_iter()
        .chain(points_not_on_curve.into_iter())
        .collect::<Vec<_>>();

    let start = Instant::now();

    let display_points = points
        .iter()
        .map(|pt| {
            if let Some(projected) = curve.invert_point(*pt) {
                ModelPoint::new(0.into(), projected.pos.project(), Vec3::zero(), Rgba::GREEN)
            } else {
                ModelPoint::new(0.into(), *pt, Vec3::zero(), Rgba::RED)
            }
        })
        .collect::<Vec<_>>();

    let end = Instant::now();
    println!(
        "Projected {} points in {}μs",
        points.len(),
        (end - start).as_micros()
    );
    println!(
        "{}μs per point",
        ((end - start) / points.len() as u32).as_micros()
    );

    // Curve beziers
    for (i, bezier) in curve.straight_beziers().iter().enumerate() {
        model.add_edge(ModelEdge::from_vec3s(
            bezier.curve.tessellate_by_param(resolution),
            match i % 2 {
                0 => Rgba::RED,
                _ => Rgba::CYAN,
            },
        ));
    }

    // Projected points
    model.add_points(display_points);

    geometry.insert_model(model);

    let mut sb = SceneBuilder::empty();
    sb.background(rgba(0.05, 0.1, 0.15, 1.0))
        .camera(Camera::create_perspective(
            [0, 0],
            point3(0.0, 0.0, -5.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, -1.0, 0.0).normalize(),
            Deg(70.0).into(),
            0.01,
            5.0,
        ))
        .lights(
            Lights::empty()
                .ambient(Rgb::WHITE, 0.2)
                .directional(vec3(1.0, 0.0, 1.0).normalize(), rgb(0.0, 0.0, 1.0), 0.3)
                .directional(vec3(-1.0, 0.0, 1.0).normalize(), rgb(1.0, 1.0, 0.0), 0.3),
        )
        .geometry(geometry);

    run_viewer(sb.build());
}
