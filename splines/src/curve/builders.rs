use cgmath::{Matrix4, Rad, Zero};
use primitives::{Angle, EVec, Vec3};

use crate::{
    backward_substitution, basis, basis_derivatives, forward_substitution,
    get_interpolation_params, knots::KnotVec, lu_decomposition, Curve, Vec4,
};

const ARC_SPLIT_DEG: f64 = 90.0;

impl Curve {
    pub fn line(start: Vec3, end: Vec3) -> Curve {
        Self::unweighted(
            vec![start.to_hpoint(1.0), end.to_hpoint(1.0)],
            KnotVec::from([0.0, 0.0, 1.0, 1.0]),
        )
    }

    pub fn interpolate_ders(points_ders: Vec<(Vec4, Vec4)>, degree: usize) -> Curve {
        let n = points_ders.len();

        // Split points and derivatives
        let mut points = Vec::new();
        let mut ders = Vec::new();
        for (point, der) in points_ders.into_iter() {
            points.push(point);
            ders.push(der);
        }

        // Compute params
        let params = get_interpolation_params(&points);

        // Compute knots
        let num_middle_knots = 2 * n - degree - 1;
        let mut knots = vec![0.0; degree + 1];
        for i in 0..num_middle_knots {
            if degree == 2 {
                let i = i + 1;
                let lower = (i - i % 2) / 2;
                let upper = lower + i % 2;
                let knot = (params[lower] + params[upper]) / 2.0;

                knots.push(knot);
            } else {
                unimplemented!("Knot calculation not implemented for degree {}", degree);
            }
        }
        knots.extend((0..degree + 1).map(|_| 1.0));
        let knots = KnotVec::new(knots);

        let mut coeffs: Vec<Vec<f64>> = vec![];
        for i in 0..n {
            let mut point_row = vec![0.0; 2 * n];
            let mut der_row = vec![0.0; 2 * n];
            let span = knots.find_span(degree, params[i]);
            let basis_ders = basis_derivatives(span, params[i], degree, &knots, 1);
            let start = span - degree;
            for c in start..=start + basis_ders.len() {
                point_row[c] = basis_ders[0][c - start];
                der_row[c] = basis_ders[1][c - start];
            }
            if i < n - 1 {
                coeffs.push(point_row);
                coeffs.push(der_row);
            } else {
                coeffs.push(der_row);
                coeffs.push(point_row);
            }
        }

        let decomp = lu_decomposition(coeffs);

        let mut bt: Vec<Vec4> = vec![];

        for i in 0..n {
            if i < n - 1 {
                bt.push(points[i]);
                bt.push(ders[i]);
            } else {
                bt.push(ders[i]);
                bt.push(points[i]);
            }
        }

        let mut ctrl_pts = vec![Vec4::zero(); 2 * n];
        for i in 0..4 {
            let bt = bt.iter().map(|x| x[i]).collect::<Vec<f64>>();

            let y = forward_substitution(&decomp.lower, bt);
            let xt = backward_substitution(&decomp.upper, y);

            for j in 0..ctrl_pts.len() {
                ctrl_pts[j][i] = xt[j];
            }
        }

        Self::unweighted(ctrl_pts, knots)
    }

    pub fn interpolate(points: Vec<Vec4>, degree: usize) -> Curve {
        let params = get_interpolation_params(&points);
        Self::interpolate_with_params(points, degree, &params)
    }

    pub fn interpolate_with_params(points: Vec<Vec4>, degree: usize, params: &[f64]) -> Curve {
        let n = points.len();

        // Compute knot vector (Eq. 9.8, The NURBS Book)
        let mut knots = vec![0.0; degree + 1];
        for i in 0..n - degree - 1 {
            knots.push(
                (1.0 / degree as f64) * (i + 1..i + degree + 1).map(|j| params[j]).sum::<f64>(),
            );
        }
        knots.extend((0..degree + 1).map(|_| 1.0));
        let knots = KnotVec::new(knots);

        let mut coeffs = vec![vec![0.0; n]; n];
        for i in 0..n {
            let span = knots.find_span(degree, params[i]);
            let new_coeffs = basis(span, params[i], degree, &knots);
            let start = span - degree;
            for c in start..start + new_coeffs.len() {
                coeffs[i][c] = new_coeffs[c - start];
            }
        }

        let decomp = lu_decomposition(coeffs);

        let mut ctrl_pts = vec![Vec4::new(0.0, 0.0, 0.0, 0.0); points.len()];

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

        Self::unweighted(full_points, KnotVec::new(full_knots))
    }

    fn arc_section(angle: Angle) -> Curve {
        let half_angle = angle / 2.0;

        Curve::unweighted(
            vec![
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(1.0, half_angle.tan(), 0.0, half_angle.cos()),
                Vec4::new(angle.cos(), angle.sin(), 0.0, 1.0),
            ],
            KnotVec::from([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
        )
    }
}
