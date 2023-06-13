use cgmath::{point3, vec3, Deg, InnerSpace, Vector3, Zero};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint},
    rgba,
    scene::SceneBuilder,
    Rgba,
};
use splines::{Pt4, SplineHelpers3, SplineHelpers4, Surface};
use viewer::run_viewer;

fn main() {
    let w = 2f64.sqrt() / 2.0;
    let surface = Surface::new(
        vec![
            vec![
                Pt4::new(1.0, 0.0, 0.0, 1.0),
                Pt4::new(1.0, 1.0, 0.0, w),
                Pt4::new(0.0, 1.0, 0.0, 1.0),
                Pt4::new(-1.0, 1.0, 0.0, w),
                Pt4::new(-1.0, 0.0, 0.0, 1.0),
                Pt4::new(-1.0, -1.0, 0.0, w),
                Pt4::new(0.0, -1.0, 0.0, 1.0),
                Pt4::new(1.0, -1.0, 0.0, w),
                Pt4::new(1.0, 0.0, 0.0, 1.0),
            ],
            vec![
                Pt4::new(1.0, 0.0, 4.0, 1.0),
                Pt4::new(1.0, 1.0, 4.0, w),
                Pt4::new(0.0, 1.0, 4.0, 1.0),
                Pt4::new(-1.0, 1.0, 4.0, w),
                Pt4::new(-1.0, 0.0, 4.0, 1.0),
                Pt4::new(-1.0, -1.0, 4.0, w),
                Pt4::new(0.0, -1.0, 4.0, 1.0),
                Pt4::new(1.0, -1.0, 4.0, w),
                Pt4::new(1.0, 0.0, 4.0, 1.0),
            ],
        ],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![
            0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
        ],
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
