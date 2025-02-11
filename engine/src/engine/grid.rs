//! A grid of strategy variants, all started on the same day so that their results are comparable.

use crate::data::panel::Panel;
use crate::engine::backtest::{run, Config, Mode, Outcome};
use crate::signal::{self, Signal};

pub struct Variant {
    pub kind: String,
    pub lookback: usize,
    pub skip: usize,
    pub rebalance: usize,
}

impl Variant {
    pub fn signal(&self) -> Result<Box<dyn Signal>, String> {
        signal::build(&self.kind, self.lookback, self.skip)
    }

    pub fn label(&self) -> String {
        format!("{}_{}_{}_{}", self.kind, self.lookback, self.skip, self.rebalance)
    }
}

pub struct GridSpec {
    pub momentum: Vec<usize>,
    pub skips: Vec<usize>,
    pub reversal: Vec<usize>,
    pub lowvol: Vec<usize>,
    pub rebalances: Vec<usize>,
}

impl GridSpec {
    pub fn variants(&self) -> Vec<Variant> {
        let mut out = Vec::new();
        for &rebalance in &self.rebalances {
            for &lookback in &self.momentum {
                for &skip in &self.skips {
                    out.push(Variant { kind: "momentum".into(), lookback, skip, rebalance });
                }
            }
            for &lookback in &self.reversal {
                out.push(Variant { kind: "reversal".into(), lookback, skip: 0, rebalance });
            }
            for &lookback in &self.lowvol {
                out.push(Variant { kind: "lowvol".into(), lookback, skip: 0, rebalance });
            }
        }
        out
    }
}

/// Largest warm-up over the variants: the common start day.
pub fn common_start(variants: &[Variant]) -> Result<usize, String> {
    let mut start = 0;
    for v in variants {
        start = start.max(v.signal()?.warmup());
    }
    Ok(start)
}

/// Runs every variant from the common start day.
pub fn run_all(panel: &Panel, variants: &[Variant], fraction: f64, mode: Mode, cost_bps: f64) -> Result<(usize, Vec<Outcome>), String> {
    let start = common_start(variants)?;
    if start + 2 >= panel.days {
        return Err("panel too short for the longest lookback".into());
    }
    let mut outcomes = Vec::with_capacity(variants.len());
    for v in variants {
        let config = Config { rebalance: v.rebalance, fraction, mode, cost_bps, start };
        outcomes.push(run(panel, v.signal()?.as_ref(), &config));
    }
    Ok((start, outcomes))
}
