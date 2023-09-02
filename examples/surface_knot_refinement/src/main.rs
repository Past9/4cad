use cgmath::{point3, vec3, vec4, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3};
use render::{
    camera::Camera,
    lights::Lights,
    model::{Geometry, Model, ModelPoint, ModelSurface},
    rgb, rgba,
    scene::SceneBuilder,
    Rgb, Rgba,
};
use splines::{Curve, KnotVec, Surface};
use tessellate::surface::SurfaceTessellation;
use viewer::run_viewer;

fn main() {
    let mut geometry = Geometry::new();
    let surface_material = geometry.insert_material(rgba(0.8, 0.8, 0.8, 1.0), 0.5);

    let mut model = Model::empty();
    let resolution = 200;

    let surface = Surface::create_unweighted(
        vec![
            vec![
                vec4(2.0, 0.0, -2.0, 1.0),
                vec4(2.0, 0.0, -1.0, 1.0),
                vec4(2.0, 0.0, 0.0, 1.0),
                vec4(2.0, 0.0, 1.0, 1.0),
                vec4(2.0, 0.0, 2.0, 1.0),
                vec4(2.0, 0.0, 3.0, 1.0),
                vec4(2.0, 0.0, 4.0, 1.0),
            ],
            vec![
                vec4(1.0, 0.0, -2.0, 1.0),
                vec4(1.0, -1.0, -1.0, 2.0),
                vec4(1.0, 0.0, 0.0, 1.0),
                vec4(1.0, 1.0, 1.0, 0.5),
                vec4(1.0, 0.0, 2.0, 1.0),
                vec4(1.0, -1.0, 3.0, 2.0),
                vec4(1.0, 0.0, 4.0, 1.0),
            ],
            vec![
                vec4(0.0, 0.0, -2.0, 1.0),
                vec4(0.0, 0.0, -1.0, 1.0),
                vec4(0.0, 0.0, 0.0, 1.0),
                vec4(0.0, 0.0, 1.0, 1.0),
                vec4(0.0, 0.0, 2.0, 1.0),
                vec4(0.0, 0.0, 3.0, 1.0),
                vec4(0.0, 0.0, 4.0, 1.0),
            ],
            vec![
                vec4(-1.0, 0.0, -2.0, 1.0),
                vec4(-1.0, 1.0, -1.0, 2.0),
                vec4(-1.0, 0.0, 0.0, 1.0),
                vec4(-1.0, -1.0, 1.0, 0.5),
                vec4(-1.0, 0.0, 2.0, 1.0),
                vec4(-1.0, 1.0, 3.0, 2.0),
                vec4(-1.0, 0.0, 4.0, 1.0),
            ],
            vec![
                vec4(-2.0, 0.0, -2.0, 1.0),
                vec4(-2.0, 0.0, -1.0, 1.0),
                vec4(-2.0, 0.0, 0.0, 1.0),
                vec4(-2.0, 0.0, 1.0, 1.0),
                vec4(-2.0, 0.0, 2.0, 1.0),
                vec4(-2.0, 0.0, 3.0, 1.0),
                vec4(-2.0, 0.0, 4.0, 1.0),
            ],
        ],
        KnotVec::from([
            // U knots
            0.0, 0.0, 0.0, 0.333, 0.666, 1.0, 1.0, 1.0,
        ]),
        KnotVec::from([
            // V knots
            0.0, 0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0, 1.0,
        ]),
    );

    model.add_surface(ModelSurface::from_surface_points(
        surface.tessellate_by_params(resolution),
        surface_material,
    ));

    for knot_u in surface.knots_u().knots().iter() {
        for knot_v in surface.knots_v().knots().iter() {
            model.add_point(ModelPoint::new(
                0.into(),
                surface.eval_pos(*knot_u, *knot_v).project(),
                Vec3::zero(),
                Rgba::GREEN,
            ));
        }
    }

    let refinements = (1..=9)
        .into_iter()
        .map(|k| k as f64 / 10.0)
        .collect::<Vec<_>>();

    // U Refined
    let refined_u_surface = surface
        .transform(&Mat4::from_translation(vec3(-4.5, 0.0, 0.0)))
        .refine_knots_u(refinements.clone());

    model.add_surface(ModelSurface::from_surface_points(
        refined_u_surface.tessellate_by_params(resolution),
        surface_material,
    ));

    geometry.insert_model(model);

    let mut sb = SceneBuilder::empty();
    sb.background(rgba(0.05, 0.1, 0.15, 1.0))
        .camera(Camera::create_perspective(
            [0, 0],
            point3(0.0, -7.0, -7.0),
            vec3(0.0, 1.0, 1.0),
            vec3(0.0, -1.0, 0.0).normalize(),
            Deg(52.0).into(),
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
