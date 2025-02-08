use backlab::portfolio::costs::{cost, turnover};
use backlab::portfolio::weights::{gross, leg_size, long_only, long_short, net};

#[test]
fn long_short_is_neutral_with_unit_gross() {
    let scores = [0.5, -1.0, 2.0, 0.1, 0.9, -0.3, 1.5, 0.0, -2.0, 0.7];
    let mut w = [0.0; 10];
    long_short(&scores, 0.2, &mut w);
    assert!(net(&w).abs() < 1e-15 && (gross(&w) - 1.0).abs() < 1e-15);
    assert_eq!((w[2], w[6]), (0.25, 0.25));
    assert_eq!((w[8], w[1]), (-0.25, -0.25));
    assert_eq!(w.iter().filter(|x| **x == 0.0).count(), 6);
}

#[test]
fn long_only_holds_the_top_names() {
    let scores = [0.5, -1.0, 2.0, 0.1, 0.9];
    let mut w = [0.0; 5];
    long_only(&scores, 0.4, &mut w);
    assert_eq!(w, [0.0, 0.0, 0.5, 0.0, 0.5]);
    assert!((net(&w) - 1.0).abs() < 1e-15);
}

#[test]
fn leg_sizes_are_sane_at_the_edges() {
    assert_eq!(leg_size(300, 0.2), 60);
    assert_eq!(leg_size(10, 0.01), 1);
    assert_eq!(leg_size(10, 0.9), 5);
    assert_eq!(leg_size(1, 0.5), 1);
}

#[test]
fn turnover_and_cost() {
    assert!((turnover(&[0.5, 0.5, 0.0], &[0.0, 0.5, 0.5]) - 1.0).abs() < 1e-15);
    assert_eq!(turnover(&[0.2, -0.2], &[0.2, -0.2]), 0.0);
    // reversing a unit-gross long-short book trades two units
    assert!((turnover(&[0.5, -0.5], &[-0.5, 0.5]) - 2.0).abs() < 1e-15);
    assert!((cost(2.0, 10.0) - 0.002).abs() < 1e-18);
}
