//! Short-term reversal: minus the return over the last `lookback` days.

use super::Signal;
use crate::data::panel::Panel;

pub struct Reversal {
    pub lookback: usize,
}

impl Signal for Reversal {
    fn name(&self) -> String {
        format!("reversal {}", self.lookback)
    }

    fn warmup(&self) -> usize {
        self.lookback
    }

    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]) {
        for (i, o) in out.iter_mut().enumerate() {
            *o = 1.0 - panel.price(day, i) / panel.price(day - self.lookback, i);
        }
    }
}
