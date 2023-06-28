use crate::{knots::KnotVec, Curve, Surface, Vec4};
use cgmath::{InnerSpace, Vector4, Zero};
use primitives::{HVec, Mat4, Vec3};

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

        Self::unweighted(
            vec![row1, row2],
            KnotVec::from([0.0, 0.0, 1.0, 1.0]),
            knots_v,
        )
    }

    pub fn generate_sweep_section_curves(
        curve: &Curve,
        trajectory: &Curve,
        mut num_sections: usize,
        scale: f64,
    ) -> (KnotVec, Vec<f64>, Vec<Curve>) {
        let q = trajectory.degree;
        let ktv = trajectory.knots.len();

        let knots_v = if ktv <= num_sections + q {
            // Refine trajectory's knot vector
            let m = num_sections + q - ktv + 1;
            trajectory.knots.split_largest_span(m)
        } else {
            // Increase the number of instances of `curve`
            if ktv > num_sections + q + 1 {
                num_sections = ktv - q - 1;
            }

            // Use trajectory's knot vector
            trajectory.knots.clone()
        };

        // Compute parameters by averaging knots
        let mut params_v = vec![0f64; num_sections];
        params_v[num_sections - 1] = 1.0;
        for k in 1..num_sections - 1 {
            params_v[k] = (1..=q).map(|i| knots_v[k + i]).sum::<f64>() / q as f64;
        }

        let mut section_curves = vec![];
        for k in 0..num_sections {
            // Transform and position section control points
            let v = params_v[k];
            let trajectory_ders = trajectory.eval_derivatives(v, 2);
            let tder1 = trajectory_ders[1].project();
            let tder2 = trajectory_ders[2].project();

            let o = trajectory_ders[0].project();

            let y = tder1.normalize();
            let z = tder1.cross(tder2).normalize();
            let x = y.cross(z);

            let mat_a = Mat4::from_translation(o)
                * Mat4::new(
                    x.x, x.y, x.z, 0.0, //
                    y.x, y.y, y.z, 0.0, //
                    z.x, z.y, z.z, 0.0, //
                    0.0, 0.0, 0.0, 1.0, //
                );

            let mut ctrl_pts = vec![Vec4::zero(); curve.unweighted.len()];
            for i in 0..curve.num_pts() {
                let pt = curve.unweighted[i];
                let transformed = mat_a.clone() * Vector4::new(pt.x, pt.y, pt.z, 1.0);
                ctrl_pts[i] = Vec4::new(transformed.x, transformed.y, transformed.z, pt.w).weight();

                ctrl_pts[i] *= trajectory_ders[0].w;
            }
            section_curves.push(Curve::weighted(ctrl_pts, curve.knots.clone()))
        }

        (knots_v, params_v, section_curves)
    }

    pub fn sweep_curve(curve: &Curve, trajectory: &Curve, num_sections: usize, scale: f64) -> Self {
        let (knots_v, params_v, section_curves) =
            Self::generate_sweep_section_curves(curve, trajectory, num_sections, scale);

        let mut curves = vec![];
        for i in 0..curve.num_pts() {
            let points: Vec<Vec4> = (0..section_curves.len())
                .map(|k| section_curves[k].weighted[i])
                .collect();
            curves.push(Curve::interpolate_with_params(
                points,
                trajectory.degree,
                &params_v,
            ));
        }

        Self::weighted(
            curves.into_iter().map(Curve::take_weighted).collect(),
            curve.knots.clone(),
            knots_v,
        )
    }

    pub fn skin_curves(curves: &[Curve], degree: usize) -> Self {
        if degree < 1 {
            panic!(
                "Skinned surface must be degree 1 or higher, but degree {} was given",
                degree
            );
        }

        if (curves.len() as i64 - degree as i64) < 1 {
            panic!(
                "At least {} curves are required to produce a degree-{} skinned surface",
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

            // Now create the curves by interpolating them through the control points
            // along the V direction, using the parameterization computed above
            (0..n)
                .map(|i| {
                    Curve::interpolate_with_params(
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
}
