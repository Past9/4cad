use cgmath::{InnerSpace, Matrix4, Rad, Zero};
use primitives::Angle;

use crate::{
    backward_substitution, basis, forward_substitution, knots::KnotVec, lu_decomposition, Curve,
    EPoint, Pt3, Pt4,
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
        let num_knots = n + degree + 1;

        // Compute param values (Eq. 9.6, The NURBS Book)
        let mut uk = vec![0.0; n];
        let d: f64 = (1..n)
            .map(|k| (points[k] - points[k - 1]).magnitude().sqrt())
            .sum();
        for k in 1..n {
            uk[k] = uk[k - 1] + ((points[k] - points[k - 1]).magnitude()).sqrt() / d;
        }

        // Compute knot vector (Eq. 9.8, The NURBS Book)
        let num_middle_knots = num_knots - degree * 2 - 2;
        let mut knots = (0..=degree)
            .map(|_| 0.0) // First degree + 1 points;
            .chain((0..num_middle_knots).map(|_| 0.0)) // Middle knots, will be overwritten
            .chain((0..=degree).map(|_| 1.0)) // Last degree + 1 knots
            .collect::<Vec<f64>>();

        for j in 1..=(n - degree - 1) {
            knots[j + degree] =
                (1.0 / degree as f64) * (j..=j + degree - 1).map(|i| uk[i]).sum::<f64>()
        }

        let knots = KnotVec::new(knots);

        let mut coeffs = vec![vec![0.0; n]; n];
        for i in 0..n {
            let span = knots.find_span(degree, uk[i]);
            //coeffs[i][span - degree] = basis(span, uk[i], degree, &knots);
            coeffs[i] = basis(span, uk[i], degree, &knots);
            println!("coeffs[{}] = {:#?}", i, coeffs[i]);
        }

        let decomp = lu_decomposition(coeffs);

        let mut ctrl_pts = vec![Pt4::zero(); points.len()];

        for i in 0..4 {
            let bt = points.iter().map(|pt| pt[i]).collect::<Vec<f64>>();
            let y = forward_substitution(&decomp.lower, bt);
            let xt = backward_substitution(&decomp.upper, y);
            for j in 0..points.len() {
                ctrl_pts[j][i] = xt[j];
            }
        }

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
