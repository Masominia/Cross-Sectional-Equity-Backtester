use backlab::math::rng::Rng;
use backlab::metrics::drawdown::{cumulative, longest_underwater, max_drawdown};
use backlab::metrics::performance::{sharpe, summarise};
use backlab::metrics::sharpe_stats::{deflated_sharpe, expected_max_sharpe, probabilistic_sharpe, sharpe_std_error};

#[test]
fn drawdowns_by_hand() {
    let r = [0.02, 0.01, -0.03, -0.01, 0.02, 0.03, -0.01];
    assert!((max_drawdown(&r) - 0.04).abs() < 1e-15);
    assert_eq!(longest_underwater(&r), 3);
    let c = cumulative(&r);
    assert!((c[2] - 0.0).abs() < 1e-15 && (c[6] - 0.03).abs() < 1e-15);
    assert_eq!(max_drawdown(&[0.01, 0.02]), 0.0);
}

#[test]
fn summary_of_a_known_series() {
    let r: Vec<f64> = (0..504).map(|i| if i % 2 == 0 { 0.011 } else { -0.009 }).collect();
    let s = summarise(&r);
    assert!((s.annual_return - 0.252).abs() < 1e-12);
    assert!((s.hit_rate - 0.5).abs() < 1e-15 && s.days == 504);
    assert!((s.sharpe - 0.001 / 0.010_009_935_5 * 252f64.sqrt()).abs() < 1e-3);
    assert!((s.t_stat - s.sharpe * 2f64.sqrt()).abs() < 1e-12);
    assert_eq!(sharpe(&[0.01; 10]), 0.0); // no variation: defined as zero, not infinity
}

#[test]
fn standard_error_matches_simulation() {
    // many independent series with a true annual Sharpe ratio of 1
    let mut rng = Rng::new(8);
    let n = 756;
    let mu = 1.0 / 252f64.sqrt() * 0.01;
    let mut sharpes = Vec::new();
    let mut claimed = 0.0;
    for _ in 0..400 {
        let r: Vec<f64> = (0..n).map(|_| mu + 0.01 * rng.normal()).collect();
        sharpes.push(sharpe(&r));
        claimed += sharpe_std_error(&r) / 400.0;
    }
    let mean = sharpes.iter().sum::<f64>() / 400.0;
    let sd = (sharpes.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / 399.0).sqrt();
    assert!((mean - 1.0).abs() < 0.1);
    assert!((claimed / sd - 1.0).abs() < 0.12, "claimed {claimed}, observed {sd}");
}

#[test]
fn the_best_of_many_needs_a_higher_bar() {
    assert_eq!(expected_max_sharpe(1, 0.25), 0.0);
    let (ten, thousand) = (expected_max_sharpe(10, 0.25), expected_max_sharpe(1000, 0.25));
    assert!(ten > 0.6 && ten < 0.9 && thousand > 1.5 && thousand < 1.75);
    // simulate: the largest of 50 standard normal draws averages about 2.25
    let mut rng = Rng::new(2);
    let avg: f64 = (0..2000).map(|_| (0..50).map(|_| rng.normal()).fold(f64::NEG_INFINITY, f64::max)).sum::<f64>() / 2000.0;
    assert!((expected_max_sharpe(50, 1.0) - avg).abs() < 0.05);

    let r: Vec<f64> = (0..1000).map(|_| 0.0006 + 0.01 * rng.normal()).collect();
    let alone = probabilistic_sharpe(&r, 0.0);
    let trials: Vec<f64> = (0..40).map(|_| 0.5 * rng.normal()).collect();
    let deflated = deflated_sharpe(&r, &trials);
    assert!(alone > deflated && deflated >= 0.0 && alone <= 1.0);
}
