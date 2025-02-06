//! Signals: a score per stock, higher meaning more attractive, from prices up to the close of day t.

pub mod combine;
pub mod lowvol;
pub mod momentum;
pub mod reversal;

use crate::data::panel::Panel;

pub trait Signal {
    fn name(&self) -> String;

    /// Number of earlier days the signal needs: `score` may be called for any day >= warmup.
    fn warmup(&self) -> usize;

    /// Writes one score per asset into `out`.
    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]);
}

/// Builds a signal from its kind and parameters, as used on the command line and in parameter grids.
pub fn build(kind: &str, lookback: usize, skip: usize) -> Result<Box<dyn Signal>, String> {
    if lookback == 0 {
        return Err("lookback must be at least one day".into());
    }
    match kind {
        "momentum" => Ok(Box::new(momentum::Momentum { lookback, skip })),
        "reversal" => Ok(Box::new(reversal::Reversal { lookback })),
        "lowvol" => {
            if lookback < 2 {
                return Err("low volatility needs at least two days".into());
            }
            Ok(Box::new(lowvol::LowVol { lookback }))
        }
        _ => Err(format!("unknown signal '{kind}': use momentum, reversal or lowvol")),
    }
}
