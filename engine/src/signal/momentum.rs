//! Price momentum: the return over `lookback` days ending `skip` days ago.

use super::Signal;
use crate::data::panel::Panel;

pub struct Momentum {
    pub lookback: usize,
    pub skip: usize,
}

impl Signal for Momentum {
    fn name(&self) -> String {
        format!("momentum {}/{}", self.lookback, self.skip)
    }

    fn warmup(&self) -> usize {
        self.lookback + self.skip
    }

    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]) {
        let (end, start) = (day - self.skip, day - self.skip - self.lookback);
        for (i, o) in out.iter_mut().enumerate() {
            *o = panel.price(end, i) / panel.price(start, i) - 1.0;
        }
    }
}
