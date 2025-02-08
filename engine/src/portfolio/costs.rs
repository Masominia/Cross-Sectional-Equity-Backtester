//! Turnover and proportional trading costs.

/// Sum of absolute weight changes: the traded value as a fraction of capital, buys and sells both counted.
pub fn turnover(current: &[f64], target: &[f64]) -> f64 {
    current.iter().zip(target).map(|(c, t)| (t - c).abs()).sum()
}

/// Cost, as a fraction of capital, of trading `turnover` at `bps` basis points per unit traded.
pub fn cost(turnover: f64, bps: f64) -> f64 {
    turnover * bps * 1e-4
}
