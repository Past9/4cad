use crate::{knots::KnotVec, Curve, HPoint, Mat4, Surface, Vec3};

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

        // Now construct a surface using the curves as rows of points in the
        // control net
        let knots_u = curves[0].knots.clone();
        let knots_v = KnotVec::uniform(curves.len(), degree);
        Self::weighted(
            curves.into_iter().map(|c| c.take_weighted()).collect(),
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

        /*
        if start.knots() != end.knots() {
            panic!(
                "Unequal knot vectors ({:?} != {:?})",
                start.knots(),
                end.knots()
            );
        }
        */

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
