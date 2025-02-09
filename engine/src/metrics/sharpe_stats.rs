//! How uncertain a Sharpe ratio is, and what the best of many trials is expected to be under pure luck.

use super::performance::{sharpe, DAYS_PER_YEAR};
use crate::math::normal;
use crate::math::stats::{kurtosis, skewness, variance};

/// Standard error of the annualised Sharpe ratio, allowing for skewness and fat tails of the daily returns.
pub fn sharpe_std_error(returns: &[f64]) -> f64 {
    let n = returns.len() as f64;
    let daily = sharpe(returns) / DAYS_PER_YEAR.sqrt();
    let inside = 1.0 - skewness(returns) * daily + 0.25 * (kurtosis(returns) - 1.0) * daily * daily;
    (inside.max(0.0) / (n - 1.0)).sqrt() * DAYS_PER_YEAR.sqrt()
}

/// Probability that the true Sharpe ratio exceeds `benchmark` (annualised), by the normal approximation.
pub fn probabilistic_sharpe(returns: &[f64], benchmark: f64) -> f64 {
    normal::cdf((sharpe(returns) - benchmark) / sharpe_std_error(returns))
}

/// Expected largest Sharpe ratio among `trials` independent strategies with no skill,
/// when the Sharpe ratios of the trials have variance `trial_variance`.
pub fn expected_max_sharpe(trials: usize, trial_variance: f64) -> f64 {
    if trials < 2 {
        return 0.0;
    }
    const EULER: f64 = 0.577_215_664_901_532_9;
    let n = trials as f64;
    trial_variance.sqrt() * ((1.0 - EULER) * normal::inverse_cdf(1.0 - 1.0 / n) + EULER * normal::inverse_cdf(1.0 - 1.0 / (n * std::f64::consts::E)))
}

/// Deflated Sharpe ratio: the probability that the true Sharpe ratio of the selected strategy is positive
/// once the expected maximum over the trials is taken as the benchmark.
pub fn deflated_sharpe(selected: &[f64], trial_sharpes: &[f64]) -> f64 {
    probabilistic_sharpe(selected, expected_max_sharpe(trial_sharpes.len(), variance(trial_sharpes)))
}
