use cgmath::{point3, vec3, Deg, InnerSpace, Point3, Transform, Zero};
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

    /*
    // HOMOGNEOUS

    let curve = Curve::unweighted(
        vec![
            Vec4::new(-4.1, -4.0, -4.0, 0.25),
            Vec4::new(-7.0, 3.0, -12.0, 1.0),
            Vec4::new(-3.0, 5.0, -8.0, 0.5),
            Vec4::new(2.0, 5.0, 4.0, 2.0),
            Vec4::new(6.0, 1.0, 12.0, 1.0),
            Vec4::new(5.0, -5.0, 8.0, 5.0),
            Vec4::new(-1.0, -8.0, -4.0, 1.0),
            Vec4::new(-5.0, -7.0, -9.0, 1.0),
            Vec4::new(-6.0, -2.0, -7.0, 5.0),
            Vec4::new(-3.0, 3.0, -8.0, 3.0),
            Vec4::new(1.0, 3.0, 10.0, 10.0),
            Vec4::new(0.1, 0.0, 0.1, 2.0),
        ],
        KnotVec::from([
            // knots
            0.0, 0.0, 0.0, 0.0, 0.1, 0.15, 0.2, 0.35, 0.6, 0.6, 0.6, 0.85, 1.0, 1.0, 1.0, 1.0,
        ]),
    );

    let points = vec![
        Vec3::new(1.0, 3.0, 4.0),
        Vec3::new(8.0, 1.0, 3.0),
        Vec3::new(4.0, 3.0, -1.0),
        Vec3::new(-3.0, -5.0, 0.0),
        Vec3::new(-4.0, 6.0, 4.0),
        Vec3::new(10.0, 9.0, -7.0),
        Vec3::new(10.0, -6.0, 5.0),
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(4.0, -2.0, 1.0),
        Vec3::new(0.0, 5.0, -6.0),
        Vec3::new(-2.0, 1.0, 0.0),
        Vec3::new(-6.0, 0.0, -2.0),
    ];
     */

    /*
    // EUCLIDEAN

    let curve = Curve::unweighted(
        vec![
            Vec4::new(-4.1, -4.0, -4.0, 1.0),
            Vec4::new(-7.0, 3.0, -12.0, 1.0),
            Vec4::new(-3.0, 5.0, -8.0, 1.0),
            Vec4::new(2.0, 5.0, 4.0, 1.0),
            Vec4::new(6.0, 1.0, 12.0, 1.0),
            Vec4::new(5.0, -5.0, 8.0, 1.0),
            Vec4::new(-1.0, -8.0, -4.0, 1.0),
            Vec4::new(-5.0, -7.0, -9.0, 1.0),
            Vec4::new(-6.0, -2.0, -7.0, 1.0),
            Vec4::new(-3.0, 3.0, -8.0, 1.0),
            Vec4::new(1.0, 3.0, 10.0, 1.0),
            Vec4::new(0.1, 0.0, 0.1, 1.0),
        ],
        KnotVec::from([
            // knots
            0.0, 0.0, 0.0, 0.0, 0.1, 0.15, 0.2, 0.35, 0.6, 0.6, 0.6, 0.85, 1.0, 1.0, 1.0, 1.0,
        ]),
    );

    let points = vec![
        Vec3::new(1.0, 3.0, 4.0),
        Vec3::new(8.0, 1.0, 3.0),
        Vec3::new(4.0, 3.0, -1.0),
        Vec3::new(-3.0, -5.0, 0.0),
        Vec3::new(-4.0, 6.0, 4.0),
        Vec3::new(10.0, 9.0, -7.0),
        Vec3::new(10.0, -6.0, 5.0),
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(4.0, -2.0, 1.0),
        Vec3::new(0.0, 5.0, -6.0),
        Vec3::new(-2.0, 1.0, 0.0),
        Vec3::new(-6.0, 0.0, -2.0),
    ];
     */

    // FLAT EUCLIDEAN

    let curve = Curve::unweighted(
        vec![
            Vec4::new(-2.0, 0.0, 0.0, 1.0),
            Vec4::new(-1.0, 1.0, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, -1.0, 0.0, 1.0),
            Vec4::new(2.0, 0.0, 0.0, 1.0),
        ],
        KnotVec::from([
            // knots
            0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0,
        ]),
    );

    let points: Vec<Vec3> = curve
        .transform(&Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0)))
        .tessellate_by_param(20)
        .into_iter()
        .take(1)
        .collect();

    /*
    let points = vec![
        Vec3::new(-1.0, -2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 2.0, 0.0),
    ];
         */

    struct Projection {
        start: Vec3,
        end: Vec3,
    }

    let projections: Vec<Projection> = points
        .iter()
        .flat_map(|pt| {
            let mut points = vec![];
            let projected = curve.project_point(*pt);
            points.push(Projection {
                start: *pt,
                end: curve.eval_pos(projected.u).project(),
            });
            points
        })
        .collect::<Vec<_>>();

    // Curve
    model.add_edge(ModelEdge::from_vec3s(
        curve.tessellate_by_param(resolution),
        Rgba::YELLOW,
    ));

    // Origin
    model.add_point(ModelPoint::new(
        0.into(),
        Vec3::zero(),
        Vec3::zero(),
        Rgba::WHITE,
    ));

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
