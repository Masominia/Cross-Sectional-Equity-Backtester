use backlab::math::normal;
use backlab::math::rng::Rng;
use backlab::math::stats::{argsort, correlation, kurtosis, mean, quantile, skewness, standardise, std_dev, variance};

#[test]
fn moments() {
    let x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    assert_eq!(mean(&x), 5.0);
    assert!((variance(&x) - 32.0 / 7.0).abs() < 1e-14);
    let mut rng = Rng::new(1);
    let z: Vec<f64> = (0..200_000).map(|_| rng.normal()).collect();
    assert!(mean(&z).abs() < 0.01 && (std_dev(&z) - 1.0).abs() < 0.01);
    assert!(skewness(&z).abs() < 0.03 && (kurtosis(&z) - 3.0).abs() < 0.06);
}

#[test]
fn ranking_and_quantiles() {
    assert_eq!(argsort(&[0.3, -1.0, 2.0, 0.3]), vec![1, 0, 3, 2]);
    let x = [5.0, 1.0, 3.0, 2.0, 4.0];
    assert_eq!((quantile(&x, 0.0), quantile(&x, 0.5), quantile(&x, 1.0)), (1.0, 3.0, 5.0));
    assert!((quantile(&x, 0.1) - 1.4).abs() < 1e-14);
}

#[test]
fn standardising_and_correlation() {
    let mut x = vec![1.0, 2.0, 3.0, 4.0];
    standardise(&mut x);
    assert!(mean(&x).abs() < 1e-15 && (std_dev(&x) - 1.0).abs() < 1e-15);
    let mut flat = vec![2.0; 5];
    standardise(&mut flat);
    assert!(flat.iter().all(|v| *v == 0.0));
    let (a, b) = ([1.0, 2.0, 3.0, 4.0], [2.0, 4.0, 6.0, 8.5]);
    assert!(correlation(&a, &b) > 0.99 && (correlation(&a, &[4.0, 3.0, 2.0, 1.0]) + 1.0).abs() < 1e-15);
}

#[test]
fn normal_distribution() {
    assert!((normal::cdf(1.959_963_984_540_054) - 0.975).abs() < 1e-14);
    assert!((normal::inverse_cdf(0.975) - 1.959_963_984_540_054).abs() < 1e-11);
    assert!((normal::cdf(0.0) - 0.5).abs() < 1e-16);
}
