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

        println!("params {:?}", params);

        // Compute knots
        let num_knots = 2 * n + degree + 1;
        let num_middle_knots = 2 * n - degree - 1;
        let mut knots = vec![0.0; degree + 1];
        for i in 0..num_middle_knots {
            let x = i.rem_euclid(2);
            if degree == 2 {
                let param_index = i / 2;
                println!("param_index {}", param_index);
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

        println!("num_knots {}", num_knots);
        println!("knots.len() {}", knots.len());
        println!("params {:?}", params);
        println!("knots {:?}", knots);

        let mut coeffs: Vec<Vec<f64>> = vec![];
        for i in 0..n {
            let mut point_row = vec![0.0; 2 * n];
            let mut der_row = vec![0.0; 2 * n];
            let span = knots.find_span(degree, params[i]);
            let basis_ders = basis_derivatives(span, params[i], degree, &knots, 1);
            println!("span {}", span);
            println!("basis_ders {:?}", basis_ders);
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

        println!("coeffs {:#?}", coeffs);

        /*
        panic!();

        let mut coeffs = vec![];

        let mut row1 = vec![0.0; 2 * n];
        row1[0] = 1.0;
        coeffs.push(row1);

        let mut row2 = vec![0.0; 2 * n];
        row2[0] = -1.0;
        row2[1] = 1.0;
        coeffs.push(row2);

        for i in 1..=2 * (n - 1) - degree {
            let span = knots.find_span(degree, params[i]);

            println!("i = {}", i);
            println!("params[i] = {}", params[i]);
            println!("span @ {} = {}", params[i], span);

            println!("# basis = {:#?}", basis(1, params[i], degree, &knots));
            println!("# basis = {:#?}", basis(2, params[i], degree, &knots));
            println!("# basis = {:#?}", basis(3, params[i], degree, &knots));
            println!("# basis = {:#?}", basis(4, params[i], degree, &knots));
            println!("# basis = {:#?}", basis(5, params[i], degree, &knots));

            println!(
                "basis = {:#?}",
                basis(span - degree, params[i], degree, &knots)
            );
            let new_coeffs = basis_derivatives(i, params[i], degree, &knots, 1);
            println!("new_coeffs {:#?}", new_coeffs);
            let start = span - degree - 1;
            let mut point_row = vec![0.0; 2 * n];
            let mut der_row = vec![0.0; 2 * n];
            for c in start..start + new_coeffs[0].len() {
                point_row[c] = new_coeffs[0][c - start]; // Point on curve
                der_row[c] = new_coeffs[1][c - start]; // Derivative on curve
            }
            coeffs.push(point_row);
            coeffs.push(der_row);
        }

        let mut row_2_last = vec![0.0; 2 * n];
        let len = row_2_last.len();
        row_2_last[len - 2] = -1.0;
        row_2_last[len - 1] = 1.0;
        coeffs.push(row_2_last);

        let mut row_last = vec![0.0; 2 * n];
        let len = row_last.len();
        row_last[len - 1] = 1.0;
        coeffs.push(row_last);

        println!("coeffs dimensions {} x {}", coeffs.len(), coeffs[0].len());
        println!("coeffs {:#?}", coeffs);

        panic!();
        */

        let decomp = lu_decomposition(coeffs);

        println!("decomp {:#?}", decomp);

        let mut bt: Vec<Vec4> = vec![];

        for i in 0..n {
            let point = points[i];
            let der = if i == 0 {
                // First point derivative
                //ders[i] * (knots[degree + 1] / degree as f64)
                ders[i]
            } else if i < n - 1 {
                // Middle point derivatives
                ders[i]
            } else {
                // Last point derivative
                //ders[i] * ((1.0 - knots[knots.len() - degree - 2]) / degree as f64)
                ders[i]
            };

            if i < n - 1 {
                bt.push(point);
                bt.push(der);
            } else {
                bt.push(der);
                bt.push(point);
            }
        }

        let mut ctrl_pts = vec![Vec4::zero(); 2 * n];
        for i in 0..4 {
            let bt = bt.iter().map(|x| x[i]).collect::<Vec<f64>>();
            /*
            bt.push(points[0][i]);
            bt.push((knots[degree + 1] / degree as f64) * ders[0][i]);

            for p in 1..=n - 2 {
                bt.push(points[p][i]);
                bt.push(ders[p][i]);
            }

            bt.push((1.0 - knots[knots.len() - 1 - degree - 1]) / degree as f64 * ders[n - 1][i]);
            bt.push(points[n - 1][i]);
            */

            println!("bt.len() {}", bt.len());
            println!("bt {:?}", bt);

            let y = forward_substitution(&decomp.lower, bt);
            let xt = backward_substitution(&decomp.upper, y);

            for j in 0..ctrl_pts.len() {
                ctrl_pts[j][i] = xt[j];
            }
        }

        println!("ctrl_pts {:#?}", ctrl_pts);
        println!("knots {:?}", knots);

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
