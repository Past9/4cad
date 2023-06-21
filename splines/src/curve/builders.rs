use cgmath::{InnerSpace, Matrix4, Rad, Zero};
use primitives::Angle;

use crate::{
    backward_substitution, basis, forward_substitution, knots::KnotVec, lu_decomposition, Curve,
    EPoint, HPoint, Pt3, Pt4, Vec3,
};

const ARC_SPLIT_DEG: f64 = 90.0;

impl Curve {
    pub fn line(start: Pt3, end: Pt3) -> Curve {
        Self::new(
            vec![start.to_hpoint(1.0), end.to_hpoint(1.0)],
            KnotVec::from([0.0, 0.0, 1.0, 1.0]),
        )
    }

    pub fn fit(points: Vec<Pt4>, degree: usize) -> Curve {
        let n = points.len();

        // Compute param values (Eq. 9.6, The NURBS Book)
        let mut chord_lens = vec![0.0; n + 1];
        chord_lens[n] = 1.0;
        for i in 1..n {
            let dist = (points[i] - points[i - 1]).magnitude();
            chord_lens[i] = dist.sqrt();
        }
        let total_chord_len: f64 = chord_lens.iter().skip(1).take(chord_lens.len() - 2).sum();

        let mut uk = vec![0.0; n];
        for i in 0..n {
            uk[i] = chord_lens.iter().take(i + 1).sum::<f64>() / total_chord_len;
        }

        // Compute knot vector (Eq. 9.8, The NURBS Book)
        let mut knots = vec![0.0; degree + 1];
        for i in 0..n - degree - 1 {
            knots.push((1.0 / degree as f64) * (i + 1..i + degree + 1).map(|j| uk[j]).sum::<f64>());
        }
        knots.extend((0..degree + 1).map(|_| 1.0));
        let knots = KnotVec::new(knots);

        println!("knots {:?}", knots);

        let mut coeffs = vec![vec![0.0; n]; n];
        for i in 0..n {
            let span = knots.find_span(degree, uk[i]);
            coeffs[i] = coeffs[i]
                .iter()
                .take(span - degree)
                .cloned()
                .chain(basis(span, uk[i], degree, &knots).into_iter())
                .collect();
        }

        /*
        // transpose coeffs
        let mut coeffs2 = vec![vec![0.0; n]; n];
        for i in 0..coeffs.len() {
            for j in 0..coeffs[0].len() {
                coeffs2[j][i] = coeffs[i][j];
            }
        }
        println!("coeffs2 = {:#?}", coeffs2);
        coeffs = coeffs2;
        */

        println!("coeffs {:#?}", coeffs);

        let decomp = lu_decomposition(coeffs);

        //println!("decomp {:#?}", decomp);

        let mut ctrl_pts = vec![Pt4::new(0.0, 0.0, 0.0, 0.0); points.len()];

        for i in 0..4 {
            println!("i {}", i);
            let bt = points.iter().map(|pt| pt[i]).collect::<Vec<f64>>();
            println!("decomp.lower {:#?}", decomp.lower);
            println!("bt {:#?}", bt);
            let y = forward_substitution(&decomp.lower, bt);
            println!("decomp.upper {:#?}", decomp.upper);
            println!("y {:#?}", y);
            let xt = backward_substitution(&decomp.upper, y);
            println!("xt {:?}", xt);
            for j in 0..points.len() {
                println!("j {}", j);
                ctrl_pts[j][i] = xt[j];
            }
        }

        println!("ctrl_pts {:#?}", ctrl_pts);

        Self::weighted(ctrl_pts, knots)
    }

    pub fn arc(angle: Angle) -> Curve {
        let mut full_points = vec![];
        let mut full_knots = vec![0.0, 0.0, 0.0];

        let num_sections = (angle / Angle::deg(ARC_SPLIT_DEG)).ceil().abs();
        let section_angle = angle / num_sections;

        for s in 0..num_sections as usize {
            let start_angle = section_angle * s as f64;
            let section = Self::arc_section(section_angle)
                .transform(&Matrix4::from_angle_z(Rad(start_angle.0)));

            if s == 0 {
                full_points.extend(section.unweighted);
            } else {
                let knot = (s) as f64 / num_sections as f64;
                full_knots.extend([knot, knot]);
                full_points.extend(section.unweighted.into_iter().skip(1));
            }
        }

        full_knots.extend([1.0, 1.0, 1.0]);

        Self::new(full_points, KnotVec::new(full_knots))
    }

    fn arc_section(angle: Angle) -> Curve {
        let half_angle = angle / 2.0;

        Curve::new(
            vec![
                Pt4::new(1.0, 0.0, 0.0, 1.0),
                Pt4::new(1.0, half_angle.tan(), 0.0, half_angle.cos()),
                Pt4::new(angle.cos(), angle.sin(), 0.0, 1.0),
            ],
            KnotVec::from([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
        )
    }
}
