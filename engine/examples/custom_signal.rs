//! Adding a signal: implement the trait, pass it to `run`.
//!
//!     cargo run --release --example custom_signal

use backlab::engine::backtest::{run, Config, Mode};
use backlab::math::rng::Rng;
use backlab::metrics::performance::sharpe;
use backlab::{Panel, Signal};

/// Distance of the price below its highest close of the last `lookback` days: buy what has fallen furthest.
struct BelowHigh {
    lookback: usize,
}

impl Signal for BelowHigh {
    fn name(&self) -> String {
        format!("below {}-day high", self.lookback)
    }

    fn warmup(&self) -> usize {
        self.lookback
    }

    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]) {
        for (i, o) in out.iter_mut().enumerate() {
            let high = (day - self.lookback..=day).map(|t| panel.price(t, i)).fold(f64::MIN, f64::max);
            *o = 1.0 - panel.price(day, i) / high;
        }
    }
}

fn main() {
    let (days, assets) = (800, 40);
    let mut rng = Rng::new(5);
    let mut prices = vec![100.0; assets];
    let mut table = prices.clone();
    let mut shock = vec![0.0; assets];
    for _ in 1..days {
        for (p, s) in prices.iter_mut().zip(shock.iter_mut()) {
            // each day's move partly undoes the previous one: a market with short-term reversal
            let fresh = 0.02 * rng.normal();
            *p *= 1.0 + fresh - 0.04 * *s;
            *s = fresh;
        }
        table.extend_from_slice(&prices);
    }
    let panel = Panel::new((0..assets).map(|i| format!("S{i}")).collect(), table).unwrap();
    let signal = BelowHigh { lookback: 5 };
    let out = run(&panel, &signal, &Config { rebalance: 1, fraction: 0.2, mode: Mode::LongShort, cost_bps: 5.0, start: 5 });
    println!("{}: gross Sharpe {:.2}, net {:.2}, turnover {:.2} per day", signal.name(), sharpe(&out.gross), sharpe(&out.net), out.turnover);
}
