//! Rank-based equal-weight portfolios.

use crate::math::stats::argsort;

/// Number of names in one leg for a universe of `assets` and a fraction such as 0.2.
pub fn leg_size(assets: usize, fraction: f64) -> usize {
    ((assets as f64 * fraction).round() as usize).clamp(1, (assets / 2).max(1))
}

/// Long the top `fraction` of scores, short the bottom `fraction`, equal weights.
/// The long leg sums to +0.5 and the short leg to -0.5: no net exposure, one unit of gross exposure.
pub fn long_short(scores: &[f64], fraction: f64, out: &mut [f64]) {
    let k = leg_size(scores.len(), fraction);
    let order = argsort(scores);
    out.iter_mut().for_each(|w| *w = 0.0);
    for &i in &order[..k] {
        out[i] = -0.5 / k as f64;
    }
    for &i in &order[order.len() - k..] {
        out[i] = 0.5 / k as f64;
    }
}

/// Long the top `fraction` of scores with equal weights that sum to one.
pub fn long_only(scores: &[f64], fraction: f64, out: &mut [f64]) {
    let k = leg_size(scores.len(), fraction);
    let order = argsort(scores);
    out.iter_mut().for_each(|w| *w = 0.0);
    for &i in &order[order.len() - k..] {
        out[i] = 1.0 / k as f64;
    }
}

pub fn gross(weights: &[f64]) -> f64 {
    weights.iter().map(|w| w.abs()).sum()
}

pub fn net(weights: &[f64]) -> f64 {
    weights.iter().sum()
}
