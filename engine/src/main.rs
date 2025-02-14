//! Command line front end: `backlab <command> <prices.csv> key=value ...`, output as `key value` and `row ...` lines.
//!
//! Commands: describe, run, sweep, walkforward.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;

use backlab::data::csv;
use backlab::engine::backtest::{run, Config, Mode};
use backlab::engine::grid::{run_all, GridSpec, Variant};
use backlab::engine::walkforward::walk_forward;
use backlab::inference::bootstrap::sharpe_interval;
use backlab::inference::permutation::{null_max_sharpes, null_sharpes, p_value};
use backlab::math::stats::{correlation, mean, quantile, variance};
use backlab::metrics::drawdown::cumulative;
use backlab::metrics::performance::{sharpe, summarise};
use backlab::metrics::sharpe_stats::{deflated_sharpe, expected_max_sharpe, probabilistic_sharpe, sharpe_std_error};
use backlab::signal;
use backlab::Panel;

fn fail(msg: &str) -> ! {
    eprintln!("backlab: {msg}");
    process::exit(1)
}

struct Args(HashMap<String, String>);

impl Args {
    fn num(&self, key: &str, default: f64) -> f64 {
        self.0.get(key).map_or(default, |v| v.parse().unwrap_or_else(|_| fail(&format!("{key} is not a number"))))
    }
    fn int(&self, key: &str, default: usize) -> usize {
        self.num(key, default as f64) as usize
    }
    fn text(&self, key: &str, default: &str) -> String {
        self.0.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
    fn list(&self, key: &str, default: &str) -> Vec<usize> {
        let text = self.text(key, default);
        if text.is_empty() || text == "none" {
            return Vec::new();
        }
        text.split(',').map(|s| s.parse().unwrap_or_else(|_| fail(&format!("{key} must be a list of integers")))).collect()
    }
    fn mode(&self) -> Mode {
        Mode::parse(&self.text("mode", "long_short")).unwrap_or_else(|| fail("mode must be long_short or long_only"))
    }
    fn grid(&self) -> Vec<Variant> {
        let spec = GridSpec {
            momentum: self.list("momentum", "20,60,120,250"),
            skips: self.list("skips", "0,20"),
            reversal: self.list("reversal", "1,2,5,10,20"),
            lowvol: self.list("lowvol", "20,60,120,250"),
            rebalances: self.list("rebalances", "5,20"),
        };
        let variants = spec.variants();
        if variants.is_empty() {
            fail("the grid is empty");
        }
        variants
    }
}

fn load(path: &str, a: &Args) -> Panel {
    let panel = csv::read(path).unwrap_or_else(|e| fail(&e));
    match a.0.get("assets") {
        Some(_) => panel.truncated(a.int("assets", panel.assets)),
        None => panel,
    }
}

fn print_summary(prefix: &str, returns: &[f64]) {
    let s = summarise(returns);
    println!("{prefix}_return {:.10}\n{prefix}_volatility {:.10}\n{prefix}_sharpe {:.10}\n{prefix}_t_stat {:.10}", s.annual_return, s.annual_volatility, s.sharpe, s.t_stat);
    println!("{prefix}_max_drawdown {:.10}\n{prefix}_longest_underwater {}\n{prefix}_skewness {:.10}\n{prefix}_kurtosis {:.10}\n{prefix}_hit_rate {:.10}", s.max_drawdown, s.longest_underwater, s.skewness, s.kurtosis, s.hit_rate);
}

fn main() {
    let mut argv = env::args().skip(1);
    let command = argv.next().unwrap_or_else(|| fail("usage: backlab <describe|run|sweep|walkforward> <prices.csv> key=value ..."));
    let path = argv.next().unwrap_or_else(|| fail("missing the price file"));
    let a = Args(argv.filter_map(|s| s.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect());
    let panel = load(&path, &a);
    let (fraction, mode, cost_bps, seed) = (a.num("fraction", 0.2), a.mode(), a.num("cost_bps", 10.0), a.int("seed", 1) as u64);
    match command.as_str() {
        "describe" => {
            let bench = panel.equal_weight_returns();
            println!("days {}\nassets {}", panel.days, panel.assets);
            print_summary("benchmark", &bench[1..]);
        }
        "run" => {
            let sig = signal::build(&a.text("signal", "momentum"), a.int("lookback", 120), a.int("skip", 20)).unwrap_or_else(|e| fail(&e));
            let start = a.int("start", sig.warmup()).max(sig.warmup());
            if start + 2 >= panel.days {
                fail("panel too short for this lookback");
            }
            let config = Config { rebalance: a.int("rebalance", 20), fraction, mode, cost_bps, start };
            let out = run(&panel, sig.as_ref(), &config);
            println!("signal {}\nstart {start}\ndays {}\nrebalances {}\nturnover {:.10}", sig.name().replace(' ', "_"), out.gross.len(), out.rebalances, out.turnover);
            print_summary("gross", &out.gross);
            print_summary("net", &out.net);
            println!("gross_sharpe_std_error {:.10}\ngross_probability_positive {:.10}", sharpe_std_error(&out.gross), probabilistic_sharpe(&out.gross, 0.0));
            let draws = a.int("draws", 500);
            if draws > 0 {
                let null = null_sharpes(&panel, &config, draws, seed);
                println!("null_mean {:.10}\nnull_std {:.10}\nnull_p_value {:.10}", mean(&null), variance(&null).sqrt(), p_value(sharpe(&out.gross), &null));
            }
            let boot = a.int("bootstrap", 1000);
            if boot > 0 {
                let ci = sharpe_interval(&out.net, a.num("block", 20.0), boot, 0.95, seed);
                println!("net_sharpe_lower {:.10}\nnet_sharpe_upper {:.10}\nnet_share_not_positive {:.10}", ci.lower, ci.upper, ci.share_not_positive);
            }
            if let Some(file) = a.0.get("out") {
                let mut w = BufWriter::new(File::create(file).unwrap_or_else(|_| fail("cannot write the output file")));
                writeln!(w, "day,gross,net").unwrap();
                for (i, (g, n)) in cumulative(&out.gross).iter().zip(cumulative(&out.net)).enumerate() {
                    writeln!(w, "{},{g},{n}", start + 1 + i).unwrap();
                }
            }
        }
        "sweep" => {
            let variants = a.grid();
            let (start, outcomes) = run_all(&panel, &variants, fraction, mode, cost_bps).unwrap_or_else(|e| fail(&e));
            let draws = a.int("draws", 300);
            let gross: Vec<f64> = outcomes.iter().map(|o| sharpe(&o.gross)).collect();
            // the null depends on the construction only, so one null per rebalancing interval serves all its variants
            let mut nulls: HashMap<usize, Vec<f64>> = HashMap::new();
            let config = |rebalance: usize| Config { rebalance, fraction, mode, cost_bps, start };
            for v in &variants {
                nulls.entry(v.rebalance).or_insert_with(|| null_sharpes(&panel, &config(v.rebalance), draws, seed + v.rebalance as u64));
            }
            println!("start {start}\ndays {}\nvariants {}", outcomes[0].gross.len(), variants.len());
            for (i, v) in variants.iter().enumerate() {
                let (g, n) = (summarise(&outcomes[i].gross), summarise(&outcomes[i].net));
                println!(
                    "row variant {} {} {} {} {} {:.8} {:.8} {:.8} {:.8} {:.8} {:.8} {:.8}",
                    v.label(), v.kind, v.lookback, v.skip, v.rebalance, g.sharpe, n.sharpe, outcomes[i].turnover, g.annual_return, n.annual_return, n.max_drawdown,
                    p_value(g.sharpe, &nulls[&v.rebalance])
                );
            }
            let best = (0..gross.len()).max_by(|&i, &j| gross[i].partial_cmp(&gross[j]).unwrap()).unwrap();
            let configs: Vec<Config> = variants.iter().map(|v| config(v.rebalance)).collect();
            let max_null = null_max_sharpes(&panel, &configs, draws, seed + 1000);
            let single = &nulls[&variants[best].rebalance];
            let mut corr = Vec::new();
            for i in 0..outcomes.len() {
                for j in 0..i {
                    corr.push(correlation(&outcomes[i].gross, &outcomes[j].gross));
                }
            }
            for x in single {
                println!("row null_single {x:.6}");
            }
            for x in &max_null {
                println!("row null_max {x:.6}");
            }
            println!("best {}\nbest_gross_sharpe {:.10}\nbest_net_sharpe {:.10}", variants[best].label(), gross[best], sharpe(&outcomes[best].net));
            println!("best_std_error {:.10}\nbest_naive_p {:.10}", sharpe_std_error(&outcomes[best].gross), 1.0 - probabilistic_sharpe(&outcomes[best].gross, 0.0));
            println!("best_single_null_p {:.10}\nbest_max_null_p {:.10}", p_value(gross[best], single), p_value(gross[best], &max_null));
            println!("single_null_mean {:.10}\nsingle_null_95 {:.10}\nmax_null_mean {:.10}\nmax_null_95 {:.10}", mean(single), quantile(single, 0.95), mean(&max_null), quantile(&max_null, 0.95));
            println!("expected_max_sharpe {:.10}\ndeflated_sharpe {:.10}", expected_max_sharpe(gross.len(), variance(&gross)), deflated_sharpe(&outcomes[best].gross, &gross));
            println!("mean_abs_correlation {:.10}\npositive_variants {}", mean(&corr.iter().map(|c| c.abs()).collect::<Vec<_>>()), gross.iter().filter(|s| **s > 0.0).count());
        }
        "walkforward" => {
            let variants = a.grid();
            let (start, outcomes) = run_all(&panel, &variants, fraction, mode, cost_bps).unwrap_or_else(|e| fail(&e));
            let series: Vec<Vec<f64>> = outcomes.into_iter().map(|o| o.net).collect();
            let (train, step) = (a.int("train", 378), a.int("step", 63));
            if train + step > series[0].len() {
                fail("training window and block exceed the available days");
            }
            let wf = walk_forward(&series, train, step);
            for b in &wf.blocks {
                println!("row block {} {} {:.8} {:.8}", start + 1 + b.start, variants[b.chosen].label(), b.trained, b.earned);
            }
            // every variant over the same out-of-sample days, for comparison
            let oos: Vec<f64> = series.iter().map(|s| sharpe(&s[train..])).collect();
            let full: Vec<f64> = series.iter().map(|s| sharpe(s)).collect();
            let best_full = (0..full.len()).max_by(|&i, &j| full[i].partial_cmp(&full[j]).unwrap()).unwrap();
            let selected = sharpe(&wf.returns);
            println!("blocks {}\noos_days {}", wf.blocks.len(), wf.returns.len());
            println!("selected_sharpe {selected:.10}\nselected_std_error {:.10}", sharpe_std_error(&wf.returns));
            println!("mean_trained_sharpe {:.10}", mean(&wf.blocks.iter().map(|b| b.trained).collect::<Vec<_>>()));
            println!("mean_variant_sharpe {:.10}\nmedian_variant_sharpe {:.10}\nbest_variant_sharpe {:.10}", mean(&oos), quantile(&oos, 0.5), oos.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
            println!("variants_beaten {}\nvariants {}", oos.iter().filter(|s| **s < selected).count(), oos.len());
            println!("hindsight_best {}\nhindsight_full_sharpe {:.10}\nhindsight_oos_sharpe {:.10}", variants[best_full].label(), full[best_full], oos[best_full]);
        }
        _ => fail("unknown command"),
    }
}
