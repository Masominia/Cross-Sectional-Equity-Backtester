//! One strategy through time.

use crate::data::panel::Panel;
use crate::math::rng::Rng;
use crate::portfolio::{costs, weights};
use crate::signal::Signal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Top fraction long, bottom fraction short, no net exposure.
    LongShort,
    /// Top fraction long. Returns are reported in excess of the equal-weight portfolio of all stocks.
    LongOnly,
}

impl Mode {
    pub fn parse(text: &str) -> Option<Mode> {
        match text {
            "long_short" => Some(Mode::LongShort),
            "long_only" => Some(Mode::LongOnly),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Days between two rebalances.
    pub rebalance: usize,
    /// Fraction of the universe in each leg.
    pub fraction: f64,
    pub mode: Mode,
    /// Cost in basis points per unit of traded value.
    pub cost_bps: f64,
    /// First day on which weights are set. Must be at least the signal's warm-up.
    pub start: usize,
}

pub struct Outcome {
    /// Daily returns before costs, one per day from `start + 1` to the end of the panel.
    pub gross: Vec<f64>,
    /// The same after trading costs, charged on the day after each rebalance.
    pub net: Vec<f64>,
    /// Mean traded value per rebalance, as a fraction of capital (the first trade from cash is left out).
    pub turnover: f64,
    pub rebalances: usize,
}

/// Where the scores come from: a signal, or random numbers for the null distribution.
pub enum Source<'a> {
    Signal(&'a dyn Signal),
    Random(&'a mut Rng),
}

pub fn run(panel: &Panel, signal: &dyn Signal, config: &Config) -> Outcome {
    assert!(config.start >= signal.warmup(), "start day is inside the signal's warm-up");
    simulate(panel, Source::Signal(signal), config)
}

/// The same construction with scores drawn at random at every rebalance.
pub fn run_random(panel: &Panel, rng: &mut Rng, config: &Config) -> Outcome {
    simulate(panel, Source::Random(rng), config)
}

pub fn simulate(panel: &Panel, mut source: Source, config: &Config) -> Outcome {
    assert!(config.rebalance >= 1 && config.start + 1 < panel.days, "nothing to simulate");
    let n = panel.assets;
    let (mut scores, mut target, mut held) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    let days = panel.days - config.start - 1;
    let (mut gross, mut net) = (Vec::with_capacity(days), Vec::with_capacity(days));
    let (mut traded, mut rebalances) = (0.0, 0usize);
    for t in config.start..panel.days - 1 {
        let mut charge = 0.0;
        if (t - config.start) % config.rebalance == 0 {
            match &mut source {
                Source::Signal(s) => s.score(panel, t, &mut scores),
                Source::Random(rng) => scores.iter_mut().for_each(|x| *x = rng.uniform()),
            }
            match config.mode {
                Mode::LongShort => weights::long_short(&scores, config.fraction, &mut target),
                Mode::LongOnly => weights::long_only(&scores, config.fraction, &mut target),
            }
            let turn = costs::turnover(&held, &target);
            charge = costs::cost(turn, config.cost_bps);
            if rebalances > 0 {
                traded += turn;
            }
            rebalances += 1;
            held.copy_from_slice(&target);
        }
        let r = panel.returns_on(t + 1);
        let pnl: f64 = held.iter().zip(r).map(|(w, x)| w * x).sum();
        let reported = match config.mode {
            Mode::LongShort => pnl,
            Mode::LongOnly => pnl - r.iter().sum::<f64>() / n as f64,
        };
        gross.push(reported);
        net.push(reported - charge);
        // let the positions drift with prices until the next rebalance
        match config.mode {
            Mode::LongShort => held.iter_mut().zip(r).for_each(|(w, x)| *w *= 1.0 + x),
            Mode::LongOnly => held.iter_mut().zip(r).for_each(|(w, x)| *w *= (1.0 + x) / (1.0 + pnl)),
        }
    }
    Outcome { gross, net, turnover: if rebalances > 1 { traded / (rebalances - 1) as f64 } else { 0.0 }, rebalances }
}
