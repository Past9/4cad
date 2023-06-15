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

    pub fn loft_surfaces(start: Curve, end: Curve) -> Self {
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
