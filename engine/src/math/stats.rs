//! Descriptive statistics and ranking.

pub fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        return f64::NAN;
    }
    x.iter().sum::<f64>() / x.len() as f64
}

pub fn variance(x: &[f64]) -> f64 {
    if x.len() < 2 {
        return f64::NAN;
    }
    let m = mean(x);
    x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (x.len() - 1) as f64
}

pub fn std_dev(x: &[f64]) -> f64 {
    variance(x).sqrt()
}

fn central(x: &[f64], k: i32) -> f64 {
    let m = mean(x);
    x.iter().map(|v| (v - m).powi(k)).sum::<f64>() / x.len() as f64
}

pub fn skewness(x: &[f64]) -> f64 {
    central(x, 3) / central(x, 2).powf(1.5)
}

/// Fourth standardised moment; three for a normal distribution.
pub fn kurtosis(x: &[f64]) -> f64 {
    central(x, 4) / central(x, 2).powi(2)
}

/// Empirical quantile with linear interpolation; sorts a copy.
pub fn quantile(x: &[f64], p: f64) -> f64 {
    let mut s = x.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in quantile input"));
    if s.is_empty() {
        return f64::NAN;
    }
    let pos = p.clamp(0.0, 1.0) * (s.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(s.len() - 1);
    s[lo] + (pos - lo as f64) * (s[hi] - s[lo])
}

/// Indices that sort `x` in ascending order; ties keep their original order.
pub fn argsort(x: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..x.len()).collect();
    idx.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).expect("no NaN in scores"));
    idx
}

/// Subtracts the mean and divides by the standard deviation, in place. A constant vector becomes zero.
pub fn standardise(x: &mut [f64]) {
    let (m, s) = (mean(x), std_dev(x));
    for v in x.iter_mut() {
        *v = if s > 0.0 { (*v - m) / s } else { 0.0 };
    }
}

pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let (mx, my) = (mean(x), mean(y));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx) * (a - mx);
        syy += (b - my) * (b - my);
    }
    sxy / (sxx * syy).sqrt()
}
