mod curve;
mod surface;

pub use curve::*;
use primitives::{rat, Int, ParamD2, Rat};
pub use surface::*;

fn normalize_knots(knots: Vec<Rat>) -> Vec<Rat> {
    let max_knot = knots[knots.len() - 1].clone();
    knots.into_iter().map(|knot| knot / &max_knot).collect()
}

fn basis_s(knots: &[Rat], j: usize, m: usize, t: &Rat) -> Rat {
    if j == 0 && t.is_zero() {
        return 1.into();
    }

    if j == knots.len() - m - 1 && t.is_one() {
        return 1.into();
    }

    let tj = &knots[j];
    let tj1 = &knots[j + 1];

    if m == 1 {
        if tj <= &t && t < tj1 {
            Rat::one()
        } else {
            Rat::zero()
        }
    } else {
        let tjm = &knots[j + m];
        let tjmsub1 = &knots[j + m - 1];

        let den1 = tjmsub1 - tj;
        let l = if den1.is_zero() {
            0.into()
        } else {
            ((t - tj) / den1) * basis_s(knots, j, m - 1, t)
        };

        let den2 = tjm - tj1;
        let r = if den2.is_zero() {
            0.into()
        } else {
            ((tjm - t) / den2) * basis_s(knots, j + 1, m - 1, t)
        };

        l + r
    }
}

fn basis_d1(knots: &[Rat], j: usize, m: usize, t: &Rat) -> Rat {
    if j == 0 && t.is_zero() {
        return 1.into();
    }

    if j == knots.len() - m - 1 && t.is_one() {
        return 1.into();
    }

    let tj = &knots[j];
    let tj1 = &knots[j + 1];

    if m == 1 {
        if tj <= &t && t < tj1 {
            Rat::one()
        } else {
            Rat::zero()
        }
    } else {
        let s = t.num();
        let h = t.den();
        let tjm = &knots[j + m];
        let tjmsub1 = &knots[j + m - 1];

        let den1 = tjmsub1 - tj;
        let l = if den1.is_zero() {
            0.into()
        } else {
            ((s - h * tj) / den1) * basis_d1(knots, j, m - 1, t)
        };

        let den2 = tjm - tj1;
        let r = if den2.is_zero() {
            0.into()
        } else {
            ((h * tjm - s) / den2) * basis_d1(knots, j + 1, m - 1, t)
        };

        l + r
    }
}

fn basis_d2(knots: &[Rat], j: usize, m: usize, t: &ParamD2) -> Rat {
    let n = t.n();
    let o = t.o();
    let rat_param = rat(o, n + o);

    if j == 0 && rat_param.is_zero() {
        return 1.into();
    }

    if j == knots.len() - m - 1 && rat_param.is_one() {
        return 1.into();
    }

    let tj = &knots[j];
    let tj1 = &knots[j + 1];

    if m == 1 {
        if tj <= &rat_param && &rat_param < tj1 {
            Rat::one()
        } else {
            Rat::zero()
        }
    } else {
        let tjm = &knots[j + m];
        let tjmsub1 = &knots[j + m - 1];

        let den1 = tjmsub1 - tj;
        let l = if den1.is_zero() {
            0.into()
        } else {
            ((o - (n + o) * tj) / den1) * basis_d2(knots, j, m - 1, t)
        };

        let den2 = tjm - tj1;
        let r = if den2.is_zero() {
            0.into()
        } else {
            (((n + o) * tjm - o) / den2) * basis_d2(knots, j + 1, m - 1, t)
        };

        l + r
    }
}

fn basis_i(knots: &[Rat], j: usize, m: usize, t: &Rat, span: usize) -> Int {
    if j == 0 && t.is_zero() {
        return 1.into();
    }

    if j == knots.len() - m - 1 && t.is_one() {
        return 1.into();
    }

    let j_int = j as Int;
    let m_int = m as Int;
    let span_int = span as Int;

    let tj = &knots[j];
    let tj1 = &knots[j + 1];

    if m == 1 {
        if tj <= &t && t < tj1 {
            1
        } else {
            0
        }
    } else {
        let (s, h) = t.num_den();
        let (sj, hj) = knots[j].num_den();
        let hj1 = knots[j + 1].den();
        let (sjm, hjm) = knots[j + m].num_den();
        let hjmsub1 = knots[j + m - 1].den();

        let l_product: Int = ((span_int - m_int + 1)..=(span_int - 1))
            .filter(|b| *b != j_int - 1)
            .map(|b| {
                let b_usize = b as usize;
                let (sbm, hbm) = knots[b_usize + m].num_den();
                let (sb1, hb1) = knots[b_usize + 1].num_den();

                sbm * hb1 - sb1 * hbm
            })
            .product();

        let l = hjmsub1 * (hj * s - sj * h) * l_product * basis_i(knots, j, m - 1, t, span);

        let r_product: Int = ((span_int - m_int + 1)..=(span_int - 1))
            .filter(|b| *b != j_int)
            .map(|b| {
                let b_usize = b as usize;
                let (sbm, hbm) = knots[b_usize + m].num_den();
                let (sb1, hb1) = knots[b_usize + 1].num_den();

                sbm * hb1 - sb1 * hbm
            })
            .product();

        let r = hj1 * (sjm * h - hjm * s) * r_product * basis_i(knots, j + 1, m - 1, t, span);

        l + r
    }
}

fn knot_span(knots: &[Rat], num_pts: usize, pos: &Rat) -> usize {
    let degree = knots.len() - num_pts - 1;

    if *pos == knots[num_pts] {
        return num_pts - 1;
    }

    let mut low = degree;
    let mut high = num_pts + 1;
    let mut mid = (low + high) / 2;

    while *pos < knots[mid] || *pos >= knots[mid + 1] {
        if *pos < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }

        mid = (low + high) / 2;
    }

    return mid;
}
