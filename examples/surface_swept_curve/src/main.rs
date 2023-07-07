use std::time::Instant;

use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3, Vec4};
use render::{
    camera::Camera,
    lights::Lights,
    model::{Geometry, Model, ModelEdge, ModelPoint, ModelSurface},
    rgb, rgba,
    scene::SceneBuilder,
    Rgb, Rgba,
};
use splines::{Curve, KnotVec, Surface};
use tessellate::{curve::CurveTessellation, surface::SurfaceTessellation};
use viewer::run_viewer;

fn main() {
    let mut geometry = Geometry::new();
    let surface_material = geometry.insert_material(rgba(0.8, 0.8, 0.8, 1.0), 0.5);

    let profile = Curve::arc(Angle::deg(360.0))
        .transform(&(Mat4::from_angle_z(Deg(-90.0)) * Mat4::from_angle_y(Deg(90.0))))
        .transform(&Mat4::from_scale(1.0));

    let trajectory = Curve::arc(Angle::deg(180.0)).transform(&Mat4::from_scale(2.0));

    let num_sections = 0;
    let (_, _, sections) =
        Surface::generate_sweep_section_curves(&profile, &trajectory, num_sections);
    let surface = Surface::sweep_curve(&profile, &trajectory, num_sections);

    let outer_edge = Curve::arc(Angle::deg(360.0)).transform(&Mat4::from_scale(3.0));

    let start = Instant::now();

    let mut model = Model::empty();
    let resolution = 50;

    // Swept surface
    model.add_surface(ModelSurface::from_surface_points(
        surface.tessellate_by_params(resolution),
        surface_material,
    ));

    // Profile curve
    model.add_edge(ModelEdge::from_vec3s(
        profile.tessellate_by_param(resolution),
        Rgba::RED,
    ));

    // Trajectory curve
    model.add_edge(ModelEdge::from_vec3s(
        trajectory.tessellate_by_param(resolution),
        Rgba::GREEN,
    ));

    // Outer edge guide curve
    model.add_edge(ModelEdge::from_vec3s(
        outer_edge.tessellate_by_param(resolution * 2),
        Rgba::BLUE,
    ));

    // Section curves
    sections.iter().for_each(|s| {
        model.add_edge(ModelEdge::from_vec3s(
            s.tessellate_by_param(resolution),
            Rgba::YELLOW,
        ))
    });

    // Section curve control points
    sections
        .iter()
        .flat_map(|s| s.ref_unweighted().iter())
        .for_each(|pt| model.add_point(ModelPoint::from_vec3(pt.truncate(), Rgba::MAGENTA)));

    let end = Instant::now();
    println!(
        "Duration: {}ms",
        (end - start).as_nanos() as f64 / 1_000_000.0
    );

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
        .lights(
            Lights::empty()
                .ambient(Rgb::WHITE, 0.2)
                .directional(vec3(1.0, 0.0, 1.0).normalize(), rgb(0.0, 0.0, 1.0), 0.3)
                .directional(vec3(-1.0, 0.0, 1.0).normalize(), rgb(1.0, 1.0, 0.0), 0.3),
        )
        .geometry(geometry);

    run_viewer(sb.build());
}
