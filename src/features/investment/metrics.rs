use bigdecimal::ToPrimitive;
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use std::str::FromStr;

/// Net cash flows indexed by period 0..n
pub fn calc_npv(rate: &BigDecimal, cfs: &[BigDecimal]) -> BigDecimal {
    let one = BigDecimal::from(1);
    let mut npv = BigDecimal::zero();
    for (t, cf) in cfs.iter().enumerate() {
        let denom = pow_decimal(&(one.clone() + rate), t as u32);
        if denom.is_zero() {
            continue;
        }
        npv += cf / denom;
    }
    npv.with_scale_round(6, bigdecimal::RoundingMode::HalfUp)
}

pub fn calc_payback(cfs: &[BigDecimal]) -> Option<BigDecimal> {
    let mut cum = BigDecimal::zero();
    for (i, cf) in cfs.iter().enumerate() {
        let prev = cum.clone();
        cum += cf;
        if i > 0 && cum >= BigDecimal::zero() {
            if cf.is_zero() {
                return Some(BigDecimal::from(i as i64));
            }
            // years ≈ (i-1) + |prev|/cf
            let frac = prev.abs() / cf;
            return Some(BigDecimal::from((i - 1) as i64) + frac);
        }
    }
    None
}

pub fn calc_discounted_payback(rate: &BigDecimal, cfs: &[BigDecimal]) -> Option<BigDecimal> {
    let one = BigDecimal::from(1);
    let mut cum = BigDecimal::zero();
    for (t, cf) in cfs.iter().enumerate() {
        let denom = pow_decimal(&(one.clone() + rate), t as u32);
        let disc = if denom.is_zero() {
            cf.clone()
        } else {
            cf / denom
        };
        let prev = cum.clone();
        cum += &disc;
        if t > 0 && cum >= BigDecimal::zero() {
            if disc.is_zero() {
                return Some(BigDecimal::from(t as i64));
            }
            let frac = prev.abs() / disc;
            return Some(BigDecimal::from((t - 1) as i64) + frac);
        }
    }
    None
}

/// PI ≈ (NPV + |CF0|) / |CF0|
pub fn calc_pi(rate: &BigDecimal, cfs: &[BigDecimal]) -> Option<BigDecimal> {
    if cfs.is_empty() || cfs[0].is_zero() {
        return None;
    }
    let npv = calc_npv(rate, cfs);
    let init = cfs[0].abs();
    Some(((npv + &init) / init).with_scale_round(6, bigdecimal::RoundingMode::HalfUp))
}

/// IRR via bisection on NPV = 0. Returns None if no sign change.
pub fn calc_irr(cfs: &[BigDecimal]) -> Option<BigDecimal> {
    if cfs.len() < 2 {
        return None;
    }
    let mut lo = BigDecimal::from_str("-0.99").unwrap();
    let mut hi = BigDecimal::from(10); // 1000%
    let npv_lo = calc_npv(&lo, cfs);
    let npv_hi = calc_npv(&hi, cfs);
    if npv_lo.clone() * npv_hi.clone() > BigDecimal::zero() {
        // try expand hi
        hi = BigDecimal::from(100);
        let npv_hi2 = calc_npv(&hi, cfs);
        if npv_lo.clone() * npv_hi2 > BigDecimal::zero() {
            return None;
        }
    }
    for _ in 0..80 {
        let mid = (&lo + &hi) / BigDecimal::from(2);
        let n = calc_npv(&mid, cfs);
        if n.abs() < BigDecimal::from_str("0.000001").unwrap() {
            return Some(mid.with_scale_round(8, bigdecimal::RoundingMode::HalfUp));
        }
        if calc_npv(&lo, cfs).sign() == n.sign() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(((&lo + &hi) / BigDecimal::from(2)).with_scale_round(8, bigdecimal::RoundingMode::HalfUp))
}

/// MIRR: finance negative CFs at finance_rate, reinvest positive at reinvest_rate
pub fn calc_mirr(
    cfs: &[BigDecimal],
    finance_rate: &BigDecimal,
    reinvest_rate: &BigDecimal,
) -> Option<BigDecimal> {
    if cfs.len() < 2 {
        return None;
    }
    let n = cfs.len() - 1;
    let one = BigDecimal::from(1);
    let mut pv_neg = BigDecimal::zero();
    let mut fv_pos = BigDecimal::zero();
    for (t, cf) in cfs.iter().enumerate() {
        if *cf < BigDecimal::zero() {
            let denom = pow_decimal(&(one.clone() + finance_rate), t as u32);
            pv_neg += cf / denom;
        } else if *cf > BigDecimal::zero() {
            let mult = pow_decimal(&(one.clone() + reinvest_rate), (n - t) as u32);
            fv_pos += cf * mult;
        }
    }
    if pv_neg.is_zero() || fv_pos.is_zero() {
        return None;
    }
    // (FV_pos / |PV_neg|)^(1/n) - 1
    let ratio = fv_pos / pv_neg.abs();
    let r = ratio.to_f64()?;
    let mirr = r.powf(1.0 / n as f64) - 1.0;
    BigDecimal::from_f64(mirr).map(|x| x.with_scale_round(8, bigdecimal::RoundingMode::HalfUp))
}

fn pow_decimal(base: &BigDecimal, exp: u32) -> BigDecimal {
    let mut r = BigDecimal::from(1);
    for _ in 0..exp {
        r *= base;
    }
    r
}
