use cgmath::InnerSpace;

use crate::{get_params, knots::KnotVec, transpose, Curve, HPoint, Mat4, Surface, Vec3};

impl Surface {
    pub fn rule_curve(curve: Curve, direction: Vec3) -> Self {
        let transform = Mat4::from_translation(direction);

        let Curve {
            unweighted: row1,
            knots: knots_v,
            ..
        } = curve;

        let row2 = row1
            .clone()
            .into_iter()
            .map(|p| p.transform(&transform))
            .collect();

        Self::new(
            vec![row1, row2],
            KnotVec::from([0.0, 0.0, 1.0, 1.0]),
            knots_v,
        )
    }

    pub fn loft_curves(curves: &[Curve], degree: usize) -> Self {
        if degree < 1 {
            panic!(
                "Lofted surface must be degree 1 or higher, but degree {} was given",
                degree
            );
        }

        if (curves.len() as i64 - degree as i64) < 1 {
            panic!(
                "At least {} curves are required to produce a degree-{} lofted surface",
                degree + 1,
                degree
            );
        }

        // Elevate all the curves to the degree of the
        // maximum-degreed curve
        let max_deg = curves.iter().map(|c| c.degree()).max().unwrap();
        let mut curves = curves
            .iter()
            .map(|c| c.elevate_degree_to(max_deg))
            .collect::<Vec<_>>();

        // Refine the knots of all curves so they have identical
        // knot vectors.
        {
            // Start by merging each pair from the first pair forward.
            for i in 0..curves.len() - 1 {
                let merged_knots = curves[i].knots().merge(curves[i + 1].knots());
                curves[i] = curves[i].refine_to(&merged_knots);
                curves[i + 1] = curves[i + 1].refine_to(&merged_knots);
            }

            // Now merge each pair from the last pair backwards
            for i in (0..curves.len() - 1).rev() {
                let merged_knots = curves[i].knots().merge(curves[i + 1].knots());
                curves[i] = curves[i].refine_to(&merged_knots);
                curves[i + 1] = curves[i + 1].refine_to(&merged_knots);
            }
        }

        // We now interpolate points along the V-direction to generate control points for the surface.
        let v_curves = {
            let n = curves[0].weighted.len();

            // Calculate the total chord lengh along each new V-direction curve
            let total_chord_len = {
                let mut d = vec![];
                for i in 0..n {
                    d.push(
                        (1..curves.len())
                            .map(|k| {
                                (curves[k].weighted[i] - curves[k - 1].weighted[i]).magnitude()
                            })
                            .sum::<f64>(),
                    );
                }
                d
            };

            // Calculate the parameterization of the new curves using Eq 10.8
            let params = {
                let mut params: Vec<f64> = Vec::with_capacity(curves.len());
                params.push(0.0);
                for k in 1..curves.len() - 1 {
                    params.push(
                        params[k - 1]
                            + (0..n)
                                .map(|i| {
                                    (curves[k].weighted[i] - curves[k - 1].weighted[i]).magnitude()
                                        / total_chord_len[i]
                                })
                                .sum::<f64>()
                                / (n as f64),
                    );
                }
                params.push(1.0);
                params
            };

            // Now create the curves by fitting them to the control points along the
            // V direction, using the parameterization computed above
            (0..n)
                .map(|i| {
                    Curve::fit_with_params(
                        curves.iter().map(|curve| curve.weighted[i]).collect(),
                        degree,
                        &params,
                    )
                })
                .collect::<Vec<_>>()
        };

        // We can get the U-direction knots from the original curves
        let knots_u = curves[0].knots.clone();

        // The V-direction knots come from the new interpolated curves, which are
        // oriented along the V-direction
        let knots_v = v_curves[0].knots().clone();
        Self::weighted(
            v_curves.into_iter().map(Curve::take_weighted).collect(),
            knots_u,
            knots_v,
        )
    }

    pub fn rule_curves(start: Curve, end: Curve) -> Self {
        // Elevate the lowest-degree curve to the same degree as the other one
        let s_deg = start.degree();
        let e_deg = end.degree();
        let (start, end) = if s_deg < e_deg {
            (start.elevate_degree(e_deg - s_deg), end)
        } else if e_deg < s_deg {
            (start, end.elevate_degree(s_deg - e_deg))
        } else {
            (start, end)
        };

        if start.degree() != end.degree() {
            panic!("Unequal degrees ({} != {})", start.degree(), end.degree());
        }

        // Insert knots in each so that they have identical knot vectors
        let merged_knots = start.knots().merge(end.knots());
        let start = start.refine_to(&merged_knots);
        let end = end.refine_to(&merged_knots);

        // Construct a surface with each curve as a row of control points and
        // appropriate knot vectors, like this:
        //   Self::new(
        //      vec![points_from_start, points_from_end],
        //      vec![0.0, 0.0, 1.0, 1.0],
        //      knot_vector_from_either
        //  )
        let Curve {
            unweighted: start_points,
            knots: start_knots,
            ..
        } = start;

        let Curve {
            unweighted: end_points,
            ..
        } = end;

        Self::new(
            vec![start_points, end_points],
            KnotVec::from([0.0, 0.0, 1.0, 1.0]),
            start_knots,
        )
    }
}
