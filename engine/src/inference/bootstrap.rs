//! Stationary bootstrap of a daily return series: blocks of random length are glued together,
//! which keeps short-range dependence such as volatility clustering.

use crate::math::rng::Rng;
use crate::math::stats::quantile;
use crate::metrics::performance::sharpe;

/// One resampled series of the same length; block lengths are geometric with the given mean.
pub fn resample(returns: &[f64], mean_block: f64, rng: &mut Rng) -> Vec<f64> {
    let n = returns.len();
    let mut out = Vec::with_capacity(n);
    let mut i = rng.below(n as u64) as usize;
    for _ in 0..n {
        out.push(returns[i]);
        i = if rng.chance(1.0 / mean_block) { rng.below(n as u64) as usize } else { (i + 1) % n };
    }
    out
}

pub struct Interval {
    pub lower: f64,
    pub upper: f64,
    /// Share of resamples with a Sharpe ratio at or below zero.
    pub share_not_positive: f64,
}

/// Percentile interval for the annualised Sharpe ratio at the given confidence level.
pub fn sharpe_interval(returns: &[f64], mean_block: f64, draws: usize, level: f64, seed: u64) -> Interval {
    let mut rng = Rng::new(seed);
    let sharpes: Vec<f64> = (0..draws).map(|_| sharpe(&resample(returns, mean_block, &mut rng))).collect();
    let tail = 0.5 * (1.0 - level);
    Interval {
        lower: quantile(&sharpes, tail),
        upper: quantile(&sharpes, 1.0 - tail),
        share_not_positive: sharpes.iter().filter(|s| **s <= 0.0).count() as f64 / draws as f64,
    }
}
