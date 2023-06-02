use primitives::{rat, ParamD2, Point4d};
use render::{
    model::{model::Model, Point},
    Vec3,
};
use splines::Curve;

fn main() {
    let curve = Curve::new(
        vec![
            Point4d::new_ints(2, 0, -2, 0),
            Point4d::new_ints(1, 1, -1, 0),
            Point4d::new_ints(1, 1, 1, 0),
            Point4d::new_ints(2, 0, 2, 0),
            Point4d::new_ints(1, -1, 1, 0),
            Point4d::new_ints(1, -1, -1, 0),
            Point4d::new_ints(2, 0, -2, 0),
        ],
        vec![
            0.into(),
            0.into(),
            0.into(),
            1.into(),
            2.into(),
            2.into(),
            3.into(),
            4.into(),
            4.into(),
            4.into(),
        ],
    );

    /*
    let curve = Curve::new(
        vec![
            Point3D::new_ints(0, 0, 0).homogenize_int(1),
            Point3D::new_ints(1, 1, 0).homogenize_int(1),
        ],
        vec![0.into(), 0.into(), 2.into(), 2.into()],
    );
    */

    let mut model = Model {
        triangles: vec![],
        points: vec![],
        lines: vec![],
    };

    let num_pts = 80;
    for i in 0..=num_pts {
        let t = rat(i, num_pts) * curve.max_knot();
        println!("T {}", t);
        let no: ParamD2 = t.into();
        let p4d = curve.eval_d2(&no);
        let p3d = p4d.project();

        //println!("t @ {} = {} -> {}", t, p4d, p3d);

        model.points.push(Point {
            vertex: p3d.into(),
            expand: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        });
    }

    render::render_model(model, 10.0).unwrap();
}
