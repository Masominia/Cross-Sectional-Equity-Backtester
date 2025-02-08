mod common;

use backlab::signal::combine::Combined;
use backlab::signal::{build, Signal};
use backlab::Panel;

fn tiny() -> Panel {
    // three stocks, five days
    Panel::new(
        vec!["A".into(), "B".into(), "C".into()],
        vec![100.0, 100.0, 100.0, 110.0, 100.0, 90.0, 121.0, 100.0, 99.0, 121.0, 110.0, 99.0, 133.1, 99.0, 99.0],
    )
    .unwrap()
}

#[test]
fn momentum_by_hand() {
    let p = tiny();
    let mut s = vec![0.0; 3];
    build("momentum", 2, 0).unwrap().score(&p, 4, &mut s);
    assert!((s[0] - 0.10).abs() < 1e-12 && (s[1] + 0.01).abs() < 1e-12 && s[2].abs() < 1e-12);
    // skipping the last two days: return from day 0 to day 2
    build("momentum", 2, 2).unwrap().score(&p, 4, &mut s);
    assert!((s[0] - 0.21).abs() < 1e-12 && s[1].abs() < 1e-12 && (s[2] + 0.01).abs() < 1e-12);
    assert_eq!(build("momentum", 2, 2).unwrap().warmup(), 4);
}

#[test]
fn reversal_is_minus_the_recent_return() {
    let p = tiny();
    let (mut r, mut m) = (vec![0.0; 3], vec![0.0; 3]);
    build("reversal", 1, 0).unwrap().score(&p, 4, &mut r);
    build("momentum", 1, 0).unwrap().score(&p, 4, &mut m);
    for i in 0..3 {
        assert!((r[i] + m[i]).abs() < 1e-15);
    }
    assert!(r[1] > r[2] && r[2] > r[0]);
}

#[test]
fn low_volatility_prefers_the_quiet_stock() {
    let p = tiny();
    let mut s = vec![0.0; 3];
    build("lowvol", 4, 0).unwrap().score(&p, 4, &mut s);
    // A: returns 10%, 10%, 0%, 10%
    let a = [0.1f64, 0.1, 0.0, 0.1];
    let mean = a.iter().sum::<f64>() / 4.0;
    let sd = (a.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / 3.0).sqrt();
    assert!((s[0] + sd).abs() < 1e-12);
    assert!(s.iter().all(|x| *x <= 0.0));
}

#[test]
fn no_signal_looks_into_the_future() {
    let p = common::noise_panel(300, 12, 0.02, 4);
    let day = 200;
    let q = common::with_changed_future(&p, day, 9);
    let signals: Vec<Box<dyn Signal>> = vec![
        build("momentum", 60, 5).unwrap(),
        build("reversal", 3, 0).unwrap(),
        build("lowvol", 40, 0).unwrap(),
        Box::new(Combined { parts: vec![build("momentum", 60, 5).unwrap(), build("lowvol", 40, 0).unwrap()] }),
    ];
    for s in &signals {
        let (mut a, mut b) = (vec![0.0; 12], vec![0.0; 12]);
        s.score(&p, day, &mut a);
        s.score(&q, day, &mut b);
        assert_eq!(a, b, "{} changed when only later prices changed", s.name());
    }
}

#[test]
fn combination_and_errors() {
    let p = common::noise_panel(100, 8, 0.02, 2);
    let c = Combined { parts: vec![build("momentum", 20, 0).unwrap(), build("reversal", 20, 0).unwrap()] };
    let mut s = vec![1.0; 8];
    c.score(&p, 50, &mut s);
    // a signal and its negative cancel
    assert!(s.iter().all(|x| x.abs() < 1e-12));
    assert_eq!(c.warmup(), 20);
    assert!(c.name().contains(" + "));
    assert!(build("carry", 10, 0).is_err() && build("momentum", 0, 0).is_err() && build("lowvol", 1, 0).is_err());
}
