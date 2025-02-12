mod common;

use backlab::engine::backtest::{run, run_random, Config, Mode};
use backlab::math::rng::Rng;
use backlab::metrics::performance::sharpe;
use backlab::signal::build;
use backlab::Panel;

fn config(rebalance: usize, mode: Mode, cost_bps: f64, start: usize) -> Config {
    Config { rebalance, fraction: 0.5, mode, cost_bps, start }
}

#[test]
fn one_period_by_hand() {
    // two stocks; on day 1 A has risen and B fallen, so one-day momentum is long A and short B
    let p = Panel::new(vec!["A".into(), "B".into()], vec![100.0, 100.0, 110.0, 90.0, 121.0, 81.0, 108.9, 89.1]).unwrap();
    let out = run(&p, build("momentum", 1, 0).unwrap().as_ref(), &config(1, Mode::LongShort, 10.0, 1));
    assert_eq!(out.gross.len(), 2);
    // day 2: A +10%, B -10% -> 0.5 * 0.10 + 0.5 * 0.10
    assert!((out.gross[0] - 0.10).abs() < 1e-12);
    // day 3: A -10%, B +10%, still long A
    assert!((out.gross[1] + 0.10).abs() < 1e-12);
    // first trade from cash: one unit traded at 10 basis points
    assert!((out.gross[0] - out.net[0] - 0.001).abs() < 1e-15);
    assert_eq!(out.rebalances, 2);
}

#[test]
fn positions_drift_between_rebalances() {
    let p = Panel::new(vec!["A".into(), "B".into()], vec![100.0, 100.0, 110.0, 90.0, 121.0, 81.0, 133.1, 72.9]).unwrap();
    let out = run(&p, build("momentum", 1, 0).unwrap().as_ref(), &config(10, Mode::LongShort, 0.0, 1));
    // after day 2 the long is 0.55 and the short -0.45; day 3 repeats +10% / -10%
    assert!((out.gross[1] - (0.55 * 0.10 + 0.45 * 0.10)).abs() < 1e-12);
    assert_eq!((out.rebalances, out.turnover), (1, 0.0));
}

#[test]
fn long_only_is_reported_against_the_equal_weight_portfolio() {
    let p = Panel::new(vec!["A".into(), "B".into()], vec![100.0, 100.0, 110.0, 90.0, 121.0, 81.0]).unwrap();
    let out = run(&p, build("momentum", 1, 0).unwrap().as_ref(), &config(1, Mode::LongOnly, 0.0, 1));
    // holds A (+10%) against a benchmark that earned 0%
    assert!((out.gross[0] - 0.10).abs() < 1e-12);
}

#[test]
fn returns_do_not_depend_on_later_prices() {
    let p = common::noise_panel(260, 20, 0.02, 7);
    let cut = 180;
    let q = common::with_changed_future(&p, cut, 3);
    let s = build("momentum", 40, 5).unwrap();
    let cfg = Config { rebalance: 7, fraction: 0.2, mode: Mode::LongShort, cost_bps: 5.0, start: 45 };
    let (a, b) = (run(&p, s.as_ref(), &cfg), run(&q, s.as_ref(), &cfg));
    // returns up to and including day `cut` are entries 0 .. cut - start - 1
    let known = cut - cfg.start;
    assert_eq!(a.net[..known], b.net[..known]);
    assert_ne!(a.net[known..], b.net[known..]);
}

#[test]
fn costs_only_ever_reduce_returns_and_scale_with_turnover() {
    let p = common::noise_panel(400, 30, 0.02, 5);
    let s = build("reversal", 1, 0).unwrap();
    let cfg = Config { rebalance: 1, fraction: 0.2, mode: Mode::LongShort, cost_bps: 10.0, start: 1 };
    let out = run(&p, s.as_ref(), &cfg);
    assert!(out.gross.iter().zip(&out.net).all(|(g, n)| n <= g));
    let paid: f64 = out.gross.iter().zip(&out.net).map(|(g, n)| g - n).sum();
    let expected = (1.0 + out.turnover * (out.rebalances - 1) as f64) * 10.0 * 1e-4;
    assert!((paid - expected).abs() < 1e-12);
    assert!(out.turnover > 1.0 && out.turnover <= 2.0);
}

#[test]
fn a_real_edge_is_found_and_noise_is_not() {
    let trend = common::trending_panel(800, 40, 0.02, 0.004, 11);
    let cfg = Config { rebalance: 20, fraction: 0.2, mode: Mode::LongShort, cost_bps: 0.0, start: 120 };
    let found = sharpe(&run(&trend, build("momentum", 120, 0).unwrap().as_ref(), &cfg).gross);
    assert!(found > 1.5, "momentum on a trending panel: {found}");
    let noise = common::noise_panel(800, 40, 0.02, 11);
    let nothing = sharpe(&run(&noise, build("momentum", 120, 0).unwrap().as_ref(), &cfg).gross);
    assert!(nothing.abs() < 1.5, "momentum on noise: {nothing}");
    let mut rng = Rng::new(1);
    let random = run_random(&trend, &mut rng, &cfg);
    assert_eq!(random.gross.len(), 800 - 120 - 1);
    assert!(sharpe(&random.gross).abs() < 1.5);
}

#[test]
#[should_panic(expected = "warm-up")]
fn starting_inside_the_warm_up_is_refused() {
    let p = common::noise_panel(100, 5, 0.02, 1);
    run(&p, build("momentum", 50, 0).unwrap().as_ref(), &config(5, Mode::LongShort, 0.0, 10));
}
