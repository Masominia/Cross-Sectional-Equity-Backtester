use backlab::engine::walkforward::walk_forward;
use backlab::math::rng::Rng;
use backlab::metrics::performance::sharpe;

#[test]
fn picks_the_past_winner_and_trades_it_next() {
    // variant 0 is good in the first half, variant 1 in the second
    let n = 400;
    let a: Vec<f64> = (0..n).map(|i| if i < 200 { 0.002 } else { -0.001 } + if i % 2 == 0 { 0.001 } else { -0.001 }).collect();
    let b: Vec<f64> = (0..n).map(|i| if i < 200 { -0.001 } else { 0.002 } + if i % 2 == 0 { 0.001 } else { -0.001 }).collect();
    let wf = walk_forward(&[a.clone(), b.clone()], 50, 50);
    assert_eq!(wf.blocks.len(), 7);
    assert_eq!(wf.returns.len(), 350);
    let chosen: Vec<usize> = wf.blocks.iter().map(|x| x.chosen).collect();
    // the switch is seen only after a full window of evidence: one block late
    assert_eq!(chosen, vec![0, 0, 0, 0, 1, 1, 1]);
    assert!((wf.blocks[3].earned - a[200..250].iter().sum::<f64>()).abs() < 1e-15);
    assert_eq!(wf.blocks[0].start, 50);
}

#[test]
fn selection_among_noise_earns_nothing_despite_good_training_numbers() {
    let mut rng = Rng::new(12);
    let series: Vec<Vec<f64>> = (0..40).map(|_| (0..1500).map(|_| 0.01 * rng.normal()).collect()).collect();
    let wf = walk_forward(&series, 250, 50);
    let trained: f64 = wf.blocks.iter().map(|b| b.trained).sum::<f64>() / wf.blocks.len() as f64;
    assert!(trained > 1.2, "the best of 40 random strategies looks good in training: {trained}");
    assert!(sharpe(&wf.returns).abs() < 1.2);
}

#[test]
fn a_partial_last_block_is_kept() {
    let series = vec![vec![0.001; 130], vec![0.002; 130]];
    let wf = walk_forward(&series, 50, 30);
    assert_eq!(wf.returns.len(), 80);
    assert_eq!(wf.blocks.last().unwrap().start, 110);
}
