//! Drawdowns of a daily return series.

/// Largest peak-to-trough fall of cumulative profit, with profits added and not compounded
/// (capital is held constant, as in the long-short backtest). Returned as a positive number.
pub fn max_drawdown(returns: &[f64]) -> f64 {
    let (mut level, mut peak, mut worst) = (0.0f64, 0.0f64, 0.0f64);
    for r in returns {
        level += r;
        peak = peak.max(level);
        worst = worst.max(peak - level);
    }
    worst
}

/// Longest run of days spent below an earlier peak.
pub fn longest_underwater(returns: &[f64]) -> usize {
    let (mut level, mut peak, mut run, mut longest) = (0.0f64, 0.0f64, 0usize, 0usize);
    for r in returns {
        level += r;
        if level >= peak {
            peak = level;
            run = 0;
        } else {
            run += 1;
            longest = longest.max(run);
        }
    }
    longest
}

pub fn cumulative(returns: &[f64]) -> Vec<f64> {
    let mut level = 0.0;
    returns
        .iter()
        .map(|r| {
            level += r;
            level
        })
        .collect()
}
