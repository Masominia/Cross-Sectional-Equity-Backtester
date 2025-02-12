//! Random-portfolio null: the same universe, days, portfolio construction and rebalancing,
//! with the signal replaced by random numbers. Whatever Sharpe ratio this produces is luck by construction.

use crate::data::panel::Panel;
use crate::engine::backtest::{run_random, Config};
use crate::math::rng::Rng;
use crate::metrics::performance::sharpe;

/// Gross Sharpe ratios of `draws` random strategies built with `config`.
pub fn null_sharpes(panel: &Panel, config: &Config, draws: usize, seed: u64) -> Vec<f64> {
    let mut rng = Rng::new(seed);
    (0..draws).map(|_| sharpe(&run_random(panel, &mut rng, config).gross)).collect()
}

/// For each draw, the largest gross Sharpe ratio over one random strategy per configuration:
/// the null distribution of "the best variant of the grid".
pub fn null_max_sharpes(panel: &Panel, configs: &[Config], draws: usize, seed: u64) -> Vec<f64> {
    let mut rng = Rng::new(seed);
    (0..draws)
        .map(|_| configs.iter().map(|c| sharpe(&run_random(panel, &mut rng, c).gross)).fold(f64::NEG_INFINITY, f64::max))
        .collect()
}

/// One-sided p-value of `observed` against a simulated null, with the usual +1 correction.
pub fn p_value(observed: f64, null: &[f64]) -> f64 {
    (1 + null.iter().filter(|x| **x >= observed).count()) as f64 / (null.len() + 1) as f64
}
