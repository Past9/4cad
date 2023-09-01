use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Point3, Zero};
use primitives::{EVec, HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    lights::Lights,
    model::{Geometry, Model, ModelEdge, ModelPoint, ModelVector},
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
            0.0, 0.0, 0.0, 0.25, 0.75, 1.0, 1.0, 1.0,
        ]),
    );

    // Curve
    model.add_edge(ModelEdge::from_vec3s(
        curve.tessellate_by_param(resolution),
        Rgba::GREEN,
    ));

    // Original knots
    model.add_points(
        curve
            .knots()
            .knots()
            .into_iter()
            .map(|k| {
                ModelPoint::new(
                    0.into(),
                    curve.eval_pos(*k).project(),
                    Vec3::zero(),
                    Rgba::GREEN,
                )
            })
            .collect(),
    );

    // Refined knots
    let refinements = (1..=9)
        .into_iter()
        .map(|k| k as f64 / 10.0)
        .collect::<Vec<_>>();

    let refined_curve = curve
        .transform(&Mat4::from_translation(vec3(0.0, 0.1, 0.0)))
        .refine_knots(refinements.clone());

    model.add_edge(ModelEdge::from_vec3s(
        refined_curve.tessellate_by_param(resolution),
        Rgba::RED,
    ));

    model.add_points(
        refined_curve
            .knots()
            .knots()
            .into_iter()
            .map(|k| {
                ModelPoint::new(
                    0.into(),
                    refined_curve.eval_pos(*k).project(),
                    Vec3::zero(),
                    Rgba::RED,
                )
            })
            .collect(),
    );

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
