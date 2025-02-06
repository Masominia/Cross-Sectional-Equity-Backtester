//! Low volatility: minus the standard deviation of daily returns over `lookback` days.

use super::Signal;
use crate::data::panel::Panel;

pub struct LowVol {
    pub lookback: usize,
}

impl Signal for LowVol {
    fn name(&self) -> String {
        format!("lowvol {}", self.lookback)
    }

    fn warmup(&self) -> usize {
        self.lookback
    }

    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]) {
        let n = self.lookback as f64;
        let (mut sum, mut sum2) = (vec![0.0; out.len()], vec![0.0; out.len()]);
        for t in day + 1 - self.lookback..=day {
            for (i, r) in panel.returns_on(t).iter().enumerate() {
                sum[i] += r;
                sum2[i] += r * r;
            }
        }
        for (i, o) in out.iter_mut().enumerate() {
            let var = (sum2[i] - sum[i] * sum[i] / n) / (n - 1.0);
            *o = -var.max(0.0).sqrt();
        }
    }
}
