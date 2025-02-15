//! Why the best of many backtests proves little: the best of N strategies with no skill at all.
//!
//!     cargo run --release --example search_bias

use backlab::math::rng::Rng;
use backlab::metrics::performance::sharpe;
use backlab::metrics::sharpe_stats::expected_max_sharpe;

fn main() {
    let (days, repeats) = (1000, 200);
    let mut rng = Rng::new(3);
    println!("{:>8} {:>22} {:>12}", "trials", "best Sharpe, simulated", "formula");
    for trials in [1usize, 5, 20, 100, 500] {
        let mut total = 0.0;
        let mut all = Vec::new();
        for _ in 0..repeats {
            let mut best = f64::NEG_INFINITY;
            for _ in 0..trials {
                let r: Vec<f64> = (0..days).map(|_| 0.01 * rng.normal()).collect();
                let s = sharpe(&r);
                all.push(s);
                best = best.max(s);
            }
            total += best;
        }
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        let var = all.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (all.len() - 1) as f64;
        println!("{trials:>8} {:>22.2} {:>12.2}", total / repeats as f64, expected_max_sharpe(trials, var));
    }
}
