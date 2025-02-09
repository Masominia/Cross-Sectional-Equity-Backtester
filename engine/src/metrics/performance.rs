//! Summary of a daily return series.

use super::drawdown::{longest_underwater, max_drawdown};
use crate::math::stats::{kurtosis, mean, skewness, std_dev};

pub const DAYS_PER_YEAR: f64 = 252.0;

/// Annualised Sharpe ratio of daily returns (no risk-free rate: the series are self-financing or active returns).
pub fn sharpe(returns: &[f64]) -> f64 {
    let (m, s) = (mean(returns), std_dev(returns));
    // a constant series has no risk to divide by; rounding must not turn it into a huge ratio
    if !(s > 1e-12 * m.abs()) || s == 0.0 {
        return 0.0;
    }
    m / s * DAYS_PER_YEAR.sqrt()
}

#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub days: usize,
    pub annual_return: f64,
    pub annual_volatility: f64,
    pub sharpe: f64,
    /// Mean over its standard error: Sharpe ratio times the square root of the number of years.
    pub t_stat: f64,
    pub max_drawdown: f64,
    pub longest_underwater: usize,
    pub skewness: f64,
    pub kurtosis: f64,
    /// Share of days with a positive return.
    pub hit_rate: f64,
}

pub fn summarise(returns: &[f64]) -> Summary {
    let sr = sharpe(returns);
    Summary {
        days: returns.len(),
        annual_return: mean(returns) * DAYS_PER_YEAR,
        annual_volatility: std_dev(returns) * DAYS_PER_YEAR.sqrt(),
        sharpe: sr,
        t_stat: sr * (returns.len() as f64 / DAYS_PER_YEAR).sqrt(),
        max_drawdown: max_drawdown(returns),
        longest_underwater: longest_underwater(returns),
        skewness: skewness(returns),
        kurtosis: kurtosis(returns),
        hit_rate: returns.iter().filter(|r| **r > 0.0).count() as f64 / returns.len().max(1) as f64,
    }
}
