use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use primitives::Angle;
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};
use splines::{Curve, EPoint, HPoint, Mat4, Surface, Vec3};
use viewer::run_viewer;

fn main() {
    let curve = Curve::arc(Angle::deg(270.0));
    let surface = Surface::skin_curves(
        &vec![
            curve.clone(),
            curve.transform(&Mat4::from_translation(Vec3::new(0.0, 0.0, 4.0))),
        ],
        1,
    );

    let mut points = Vec::new();

    let num_pts = 50;
    for i in 0..=num_pts {
        for j in 0..=num_pts {
            let u = i as f64 / num_pts as f64;
            let v = j as f64 / num_pts as f64;
            let p4d = surface.eval(u, v);
            let p3d = p4d.project();

            points.push(ModelPoint::new(
                0.into(),
                p3d.as_f32(),
                Vector3::zero(),
                Rgba::WHITE,
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
