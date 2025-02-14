mod common;

use backlab::engine::backtest::{run, Config, Mode};
use backlab::inference::bootstrap::{resample, sharpe_interval};
use backlab::inference::permutation::{null_max_sharpes, null_sharpes, p_value};
use backlab::math::rng::Rng;
use backlab::math::stats::{mean, std_dev};
use backlab::metrics::performance::sharpe;
use backlab::metrics::sharpe_stats::sharpe_std_error;
use backlab::signal::build;

fn cfg(rebalance: usize) -> Config {
    Config { rebalance, fraction: 0.2, mode: Mode::LongShort, cost_bps: 0.0, start: 60 }
}

#[test]
fn p_values_by_hand() {
    let null = [0.1, 0.4, -0.2, 0.9];
    assert_eq!(p_value(1.0, &null), 0.2);
    assert_eq!(p_value(0.4, &null), 0.6);
    assert_eq!(p_value(-5.0, &null), 1.0);
}

#[test]
fn the_null_is_centred_and_as_wide_as_theory_says() {
    let p = common::noise_panel(700, 30, 0.02, 21);
    let null = null_sharpes(&p, &cfg(5), 300, 4);
    assert!(mean(&null).abs() < 0.12);
    let typical = sharpe_std_error(&run(&p, build("momentum", 60, 0).unwrap().as_ref(), &cfg(5)).gross);
    assert!((std_dev(&null) / typical - 1.0).abs() < 0.25);
    // same seed, same draws
    assert_eq!(null_sharpes(&p, &cfg(5), 20, 4), null_sharpes(&p, &cfg(5), 20, 4));
}

#[test]
fn an_edge_is_detected_and_noise_is_not() {
    let trend = common::trending_panel(700, 30, 0.02, 0.004, 5);
    let observed = sharpe(&run(&trend, build("momentum", 60, 0).unwrap().as_ref(), &cfg(20)).gross);
    assert!(p_value(observed, &null_sharpes(&trend, &cfg(20), 200, 1)) < 0.02);
    let noise = common::noise_panel(700, 30, 0.02, 5);
    let nothing = sharpe(&run(&noise, build("momentum", 60, 0).unwrap().as_ref(), &cfg(20)).gross);
    assert!(p_value(nothing, &null_sharpes(&noise, &cfg(20), 200, 1)) > 0.02);
}

#[test]
fn the_best_of_a_grid_has_a_higher_null() {
    let p = common::noise_panel(500, 20, 0.02, 9);
    let configs: Vec<Config> = (0..12).map(|_| cfg(10)).collect();
    let (single, best) = (null_sharpes(&p, &cfg(10), 150, 2), null_max_sharpes(&p, &configs, 150, 3));
    assert!(mean(&best) > mean(&single) + 2.0 * std_dev(&single) * 0.6);
    assert!(best.iter().cloned().fold(f64::INFINITY, f64::min) > mean(&single) - std_dev(&single));
}

#[test]
fn bootstrap_resamples_and_brackets_the_estimate() {
    let mut rng = Rng::new(6);
    let r: Vec<f64> = (0..1000).map(|_| 0.0005 + 0.01 * rng.normal()).collect();
    let copy = resample(&r, 20.0, &mut rng);
    assert_eq!(copy.len(), r.len());
    assert!(copy.iter().all(|x| r.contains(x)));
    let ci = sharpe_interval(&r, 20.0, 500, 0.95, 3);
    let sr = sharpe(&r);
    assert!(ci.lower < sr && sr < ci.upper);
    // the interval is about four standard errors wide
    assert!(((ci.upper - ci.lower) / (3.92 * sharpe_std_error(&r)) - 1.0).abs() < 0.25);
    assert!(ci.share_not_positive >= 0.0 && ci.share_not_positive < 0.5);
}
