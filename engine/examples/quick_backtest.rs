//! A momentum backtest on a synthetic panel, with its null distribution.
//!
//!     cargo run --release --example quick_backtest

use backlab::engine::backtest::{run, Config, Mode};
use backlab::inference::permutation::{null_sharpes, p_value};
use backlab::math::rng::Rng;
use backlab::metrics::performance::summarise;
use backlab::signal::build;
use backlab::Panel;

fn main() {
    // 60 stocks, 1,000 days, a small persistent drift that differs across stocks
    let (days, assets) = (1000, 60);
    let mut rng = Rng::new(1);
    let mut prices = vec![100.0; assets];
    let mut table = prices.clone();
    for _ in 1..days {
        for (i, p) in prices.iter_mut().enumerate() {
            *p *= 1.0 + 0.003 * (i as f64 / assets as f64 - 0.5) + 0.02 * rng.normal();
        }
        table.extend_from_slice(&prices);
    }
    let panel = Panel::new((0..assets).map(|i| format!("S{i}")).collect(), table).unwrap();

    let signal = build("momentum", 120, 0).unwrap();
    let config = Config { rebalance: 20, fraction: 0.2, mode: Mode::LongShort, cost_bps: 10.0, start: 120 };
    let out = run(&panel, signal.as_ref(), &config);
    let (g, n) = (summarise(&out.gross), summarise(&out.net));
    println!("{}: {} days, turnover {:.2} per rebalance", signal.name(), g.days, out.turnover);
    println!("gross  return {:>6.2}%  Sharpe {:>5.2}  drawdown {:>5.1}%", 100.0 * g.annual_return, g.sharpe, 100.0 * g.max_drawdown);
    println!("net    return {:>6.2}%  Sharpe {:>5.2}  drawdown {:>5.1}%", 100.0 * n.annual_return, n.sharpe, 100.0 * n.max_drawdown);
    let null = null_sharpes(&panel, &config, 500, 7);
    println!("random portfolios beat the gross Sharpe ratio with probability {:.3}", p_value(g.sharpe, &null));
}
