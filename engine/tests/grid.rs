mod common;

use backlab::engine::backtest::Mode;
use backlab::engine::grid::{common_start, run_all, GridSpec};

fn spec() -> GridSpec {
    GridSpec { momentum: vec![20, 60], skips: vec![0, 5], reversal: vec![1, 5], lowvol: vec![30], rebalances: vec![5, 20] }
}

#[test]
fn the_grid_is_the_product_of_its_lists() {
    let v = spec().variants();
    assert_eq!(v.len(), 2 * (2 * 2 + 2 + 1));
    assert_eq!(common_start(&v).unwrap(), 65);
    let labels: std::collections::HashSet<String> = v.iter().map(|x| x.label()).collect();
    assert_eq!(labels.len(), v.len());
    assert!(labels.contains("momentum_60_5_20") && labels.contains("lowvol_30_0_5"));
}

#[test]
fn all_variants_cover_the_same_days() {
    let p = common::noise_panel(300, 15, 0.02, 3);
    let v = spec().variants();
    let (start, outcomes) = run_all(&p, &v, 0.2, Mode::LongShort, 5.0).unwrap();
    assert_eq!(start, 65);
    assert!(outcomes.iter().all(|o| o.gross.len() == 300 - 65 - 1));
    // rebalancing less often trades less per unit of time
    let per_day = |i: usize| outcomes[i].turnover / v[i].rebalance as f64;
    let (fast, slow) = (v.iter().position(|x| x.label() == "reversal_1_0_5").unwrap(), v.iter().position(|x| x.label() == "reversal_1_0_20").unwrap());
    assert!(per_day(fast) > 2.0 * per_day(slow));
}

#[test]
fn a_panel_that_is_too_short_is_an_error() {
    let p = common::noise_panel(60, 5, 0.02, 3);
    assert!(run_all(&p, &spec().variants(), 0.2, Mode::LongShort, 0.0).is_err());
}
