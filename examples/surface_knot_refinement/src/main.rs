use cgmath::{point3, vec3, vec4, Deg, InnerSpace, Vector3, Zero};
use primitives::{Angle, HVec, Mat4, Vec3};
use render::{
    camera::Camera,
    model::{Geometry, Model, ModelPoint, ModelSurface},
    rgba,
    scene::SceneBuilder,
    Rgba,
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
                vec4(-2.0, 0.0, -2.0, 1.0),
                vec4(-2.0, 0.0, -1.0, 1.0),
                vec4(-2.0, 0.0, 0.0, 1.0),
                vec4(-2.0, 0.0, 1.0, 1.0),
                vec4(-2.0, 0.0, 2.0, 1.0),
            ],
            vec![
                vec4(-1.0, 0.0, -2.0, 1.0),
                vec4(-1.0, 0.0, -1.0, 1.0),
                vec4(-1.0, 0.0, 0.0, 1.0),
                vec4(-1.0, 0.0, 1.0, 1.0),
                vec4(-1.0, 0.0, 2.0, 1.0),
            ],
            vec![
                vec4(0.0, 0.0, -2.0, 1.0),
                vec4(0.0, 0.0, -1.0, 1.0),
                vec4(0.0, 0.0, 0.0, 1.0),
                vec4(0.0, 0.0, 1.0, 1.0),
                vec4(0.0, 0.0, 2.0, 1.0),
            ],
            vec![
                vec4(1.0, 0.0, -2.0, 1.0),
                vec4(1.0, 0.0, -1.0, 1.0),
                vec4(1.0, 0.0, 0.0, 1.0),
                vec4(1.0, 0.0, 1.0, 1.0),
                vec4(1.0, 0.0, 2.0, 1.0),
            ],
            vec![
                vec4(2.0, 0.0, -2.0, 1.0),
                vec4(2.0, 0.0, -1.0, 1.0),
                vec4(2.0, 0.0, 0.0, 1.0),
                vec4(2.0, 0.0, 1.0, 1.0),
                vec4(2.0, 0.0, 2.0, 1.0),
            ],
        ],
        KnotVec::from([
            // U knots
            0.0, 0.0, 0.0, 0.3, 0.7, 1.0, 1.0, 1.0,
        ]),
        KnotVec::from([
            // V knots
            0.0, 0.0, 0.0, 0.3, 0.7, 1.0, 1.0, 1.0,
        ]),
    );

    model.add_surface(ModelSurface::from_surface_points(
        surface.tessellate_by_params(resolution),
        surface_material,
    ));
}
