use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Point3, Transform, Zero};
use primitives::{Angle, EVec, HVec, Mat4, Vec3, Vec4};
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

const BEZ_OFFSET: f64 = -0.001;
const STRAIGHT_BEZ_OFFSET: f64 = BEZ_OFFSET * 2.0;

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

    let res = 500;

    let points: Vec<Vec3> = curve
        .transform(&Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0)))
        .tessellate_by_param(res)
        .into_iter()
        .chain(
            curve
                .transform(&Mat4::from_translation(Vec3::new(0.0, -0.5, 0.0)))
                .tessellate_by_param(res)
                .into_iter(),
        )
        .chain(
            curve
                .transform(&Mat4::from_translation(Vec3::new(0.0, 0.0, 0.5)))
                .tessellate_by_param(res)
                .into_iter(),
        )
        .chain(
            curve
                .transform(&Mat4::from_translation(Vec3::new(0.0, 0.0, -0.5)))
                .tessellate_by_param(res)
                .into_iter(),
        )
        .collect();

    struct Projection {
        start: Vec3,
        end: Vec3,
    }

    let start = Instant::now();
    let projections: Vec<Projection> = points
        .iter()
        .flat_map(|pt| {
            let mut points = vec![];
            if let Some(projected) = curve.nearest_point(*pt) {
                points.push(Projection {
                    start: *pt,
                    end: curve.eval_pos(projected.u).project(),
                });
            }
            points
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

    // Points
    model.add_points(
        points
            .iter()
            .map(|pt| ModelPoint::new(0.into(), *pt, Vec3::zero(), Rgba::GREEN))
            .collect(),
    );

    // Projected points
    model.add_points(
        projections
            .iter()
            .map(|projection| {
                ModelPoint::new(0.into(), projection.end.clone(), Vec3::zero(), Rgba::RED)
            })
            .collect(),
    );

    // Projection vectors
    model.add_vectors(
        projections
            .iter()
            .map(|projection| {
                ModelVector::new(
                    Point3::new(
                        projection.start.x as f32,
                        projection.start.y as f32,
                        projection.start.z as f32,
                    ),
                    (projection.end - projection.start).as_f32(),
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
