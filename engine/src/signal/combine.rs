//! An equal-weight blend of several signals after standardising each across stocks.

use super::Signal;
use crate::data::panel::Panel;
use crate::math::stats::standardise;

pub struct Combined {
    pub parts: Vec<Box<dyn Signal>>,
}

impl Signal for Combined {
    fn name(&self) -> String {
        self.parts.iter().map(|p| p.name()).collect::<Vec<_>>().join(" + ")
    }

    fn warmup(&self) -> usize {
        self.parts.iter().map(|p| p.warmup()).max().unwrap_or(0)
    }

    fn score(&self, panel: &Panel, day: usize, out: &mut [f64]) {
        let mut part = vec![0.0; out.len()];
        out.iter_mut().for_each(|o| *o = 0.0);
        for signal in &self.parts {
            signal.score(panel, day, &mut part);
            standardise(&mut part);
            for (o, p) in out.iter_mut().zip(&part) {
                *o += p / self.parts.len() as f64;
            }
        }
    }
}
