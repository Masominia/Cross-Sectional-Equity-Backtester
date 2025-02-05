//! Synthetic panels shared by the integration tests.
#![allow(dead_code)]

use backlab::math::rng::Rng;
use backlab::Panel;

/// Independent normal daily returns with the given volatility; no stock is better than another.
pub fn noise_panel(days: usize, assets: usize, vol: f64, seed: u64) -> Panel {
    let mut rng = Rng::new(seed);
    let mut prices = vec![100.0; assets];
    let mut table = prices.clone();
    for _ in 1..days {
        for p in prices.iter_mut() {
            *p *= 1.0 + vol * rng.normal();
        }
        table.extend_from_slice(&prices);
    }
    Panel::new((0..assets).map(|i| format!("S{i}")).collect(), table).unwrap()
}

/// Like `noise_panel`, but stock i has a constant daily drift proportional to its index:
/// past winners keep winning, so momentum has a real edge.
pub fn trending_panel(days: usize, assets: usize, vol: f64, drift: f64, seed: u64) -> Panel {
    let mut rng = Rng::new(seed);
    let mut prices = vec![100.0; assets];
    let mut table = prices.clone();
    for _ in 1..days {
        for (i, p) in prices.iter_mut().enumerate() {
            let mu = drift * (i as f64 / (assets - 1) as f64 - 0.5);
            *p *= 1.0 + mu + vol * rng.normal();
        }
        table.extend_from_slice(&prices);
    }
    Panel::new((0..assets).map(|i| format!("S{i}")).collect(), table).unwrap()
}

/// A copy of the panel in which every price after `day` is replaced by something else.
pub fn with_changed_future(panel: &Panel, day: usize, seed: u64) -> Panel {
    let mut rng = Rng::new(seed);
    let mut table = Vec::new();
    for t in 0..panel.days {
        for i in 0..panel.assets {
            table.push(if t <= day { panel.price(t, i) } else { panel.price(t, i) * (0.5 + rng.uniform()) });
        }
    }
    Panel::new(panel.names.clone(), table).unwrap()
}
