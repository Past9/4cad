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

const BEZ_OFFSET: f64 = 0.1;
const CONVEX_BEZ_OFFSET: f64 = BEZ_OFFSET * 2.0;
const STRAIGHT_BEZ_OFFSET: f64 = BEZ_OFFSET * 3.0;

fn main() {
    let mut geometry = Geometry::new();
    let surface_material = geometry.insert_material(rgba(0.8, 0.8, 0.8, 1.0), 0.5);
    let surface_material_alt_a =
        geometry.insert_material(rgba(0.8, 0.8, 0.8, 1.0).interpolate(Rgba::RED, 0.5), 0.5);
    let surface_material_alt_b =
        geometry.insert_material(rgba(0.8, 0.8, 0.8, 1.0).interpolate(Rgba::BLUE, 0.5), 0.5);

    let mut model = Model::empty();
    let resolution = 50;

    let nurbs = Surface::create_unweighted(
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
        nurbs.tessellate_by_params(resolution),
        surface_material,
    ));

    // U Decomposition
    let u_decomps = nurbs
        .transform(&Mat4::from_translation(vec3(-4.5, 0.0, 0.0)))
        .beziers_u()
        .to_vec();

    for (i, decomp) in u_decomps.iter().enumerate() {
        model.add_surface(ModelSurface::from_surface_points(
            decomp.surface.tessellate_by_params(resolution),
            match i % 2 == 0 {
                true => surface_material_alt_a,
                false => surface_material_alt_b,
            },
        ));
    }

    println!("u_decomps");
    for decomp in u_decomps.iter() {
        println!("{:?}, {:?}", (decomp.param_span_u), (decomp.param_span_v));
    }

    // V Decomposition
    let v_decomps = nurbs
        .transform(&Mat4::from_translation(vec3(4.5, 0.0, 0.0)))
        .beziers_v()
        .to_vec();

    for (i, decomp) in v_decomps.iter().enumerate() {
        model.add_surface(ModelSurface::from_surface_points(
            decomp.surface.tessellate_by_params(resolution),
            match i % 2 == 0 {
                true => surface_material_alt_a,
                false => surface_material_alt_b,
            },
        ));
    }

    println!("v_decomps");
    for decomp in v_decomps.iter() {
        println!("{:?}, {:?}", (decomp.param_span_u), (decomp.param_span_v));
    }

    // UV Decomposition
    let uv_decomps = nurbs
        .transform(&Mat4::from_translation(vec3(0.0, 0.0, 6.5)))
        .beziers_uv()
        .to_vec();

    println!("uv_decomps");
    for decomps in uv_decomps.iter() {
        for decomp in decomps.iter() {
            println!("{:?}, {:?}", (decomp.param_span_u), (decomp.param_span_v));
        }
    }

    for (i, row) in uv_decomps.iter().enumerate() {
        for (j, decomp) in row.iter().enumerate() {
            model.add_surface(ModelSurface::from_surface_points(
                decomp.surface.tessellate_by_params(resolution),
                match (i + j) % 2 == 0 {
                    true => surface_material_alt_a,
                    false => surface_material_alt_b,
                },
            ));
        }
    }

    geometry.insert_model(model);

    let mut sb = SceneBuilder::empty();
    sb.background(rgba(0.05, 0.1, 0.15, 1.0))
        .camera(Camera::create_perspective(
            [0, 0],
            point3(0.0, -8.0, -6.0),
            vec3(0.0, 1.1, 1.0),
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
