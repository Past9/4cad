use cgmath::{point3, vec3, Deg, InnerSpace};
use primitives::{Mat4, Vec4};
use render::{
    camera::Camera,
    lights::Lights,
    model::{Geometry, Model, ModelEdge},
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

    let nurbs = Curve::unweighted(
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

    let beziers_nurbs = nurbs.transform(&Mat4::from_translation(vec3(0.0, 0.05, 0.0)));
    let beziers = beziers_nurbs.beziers();

    let convex_beziers_nurbs = nurbs.transform(&Mat4::from_translation(vec3(0.0, 0.1, 0.0)));
    let convex_beziers = convex_beziers_nurbs.convex_beziers();

    model.add_edge(ModelEdge::from_vec3s(
        nurbs.tessellate_by_param(resolution),
        Rgba::YELLOW,
    ));

    for (i, bezier) in beziers.iter().enumerate() {
        model.add_edge(ModelEdge::from_vec3s(
            bezier.tessellate_by_param(resolution),
            match i % 2 {
                0 => Rgba::MAGENTA,
                _ => Rgba::GREEN,
            },
        ));
    }

    for (i, bezier) in convex_beziers.iter().enumerate() {
        model.add_edge(ModelEdge::from_vec3s(
            bezier.tessellate_by_param(resolution),
            match i % 2 {
                0 => Rgba::RED,
                _ => Rgba::CYAN,
            },
        ));
    }

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
