//! Walk-forward selection: at the start of each block, pick the variant with the best Sharpe ratio
//! over the preceding training window and hold it for the block.

use crate::metrics::performance::sharpe;

pub struct Block {
    /// First day of the block, as an index into the return series.
    pub start: usize,
    pub chosen: usize,
    /// Annualised Sharpe ratio of the chosen variant over the training window.
    pub trained: f64,
    /// Sum of the chosen variant's daily returns over the block.
    pub earned: f64,
}

pub struct WalkForward {
    pub blocks: Vec<Block>,
    /// Daily returns of the selected strategy, from the first block on.
    pub returns: Vec<f64>,
}

/// `series[k]` are the daily returns of variant k, all of the same length.
/// Switching between variants is not charged; each variant's own trading costs are already in its series.
pub fn walk_forward(series: &[Vec<f64>], train: usize, step: usize) -> WalkForward {
    assert!(!series.is_empty() && train >= 2 && step >= 1, "need variants, a training window and a block length");
    let days = series[0].len();
    assert!(series.iter().all(|s| s.len() == days), "all series must have the same length");
    let (mut blocks, mut returns) = (Vec::new(), Vec::new());
    let mut t = train;
    while t < days {
        let end = (t + step).min(days);
        let (mut chosen, mut best) = (0, f64::NEG_INFINITY);
        for (k, s) in series.iter().enumerate() {
            let sr = sharpe(&s[t - train..t]);
            if sr > best {
                best = sr;
                chosen = k;
            }
        }
        let block = &series[chosen][t..end];
        returns.extend_from_slice(block);
        blocks.push(Block { start: t, chosen, trained: best, earned: block.iter().sum() });
        t = end;
    }
    WalkForward { blocks, returns }
}
