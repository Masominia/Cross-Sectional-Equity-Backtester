"""All experiments, checks and the report."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pandas as pd
import yaml

from . import engine, plotting
from .engine import ROOT
from .report import md_table

VARIANT = [
    "variant",
    "signal",
    "lookback",
    "skip",
    "rebalance",
    "gross Sharpe",
    "net Sharpe",
    "turnover",
    "gross return",
    "net return",
    "net drawdown",
    "p against random",
]


def _check(checks, area, name, ok, detail, warn=False):
    checks.append(
        {"area": area, "check": name, "status": "PASS" if ok else ("WARN" if warn else "FAIL"), "detail": detail}
    )


def run(cfg_path=None, out=None, fast: bool = False) -> dict:
    cfg = yaml.safe_load(open(cfg_path or ROOT / "configs" / "default.yaml"))
    out = Path(out) if out else ROOT / "reports"
    (out / "figures").mkdir(parents=True, exist_ok=True)
    if not engine.available():
        raise SystemExit("Rust engine not built: run `make build` first")
    checks: list[dict] = []
    common = {"fraction": cfg["fraction"], "mode": cfg["mode"], "seed": cfg["seed"]}
    grid = dict(cfg["grid"])
    draws, boot = (60, 200) if fast else (cfg["draws"], cfg["bootstrap"])
    if fast:
        grid.update(momentum=[60, 120], skips=[0], reversal=[2, 5], lowvol=[60])

    desc = engine.call("describe")

    # 1. the search
    sw = engine.call("sweep", cost_bps=cfg["cost_bps"], draws=draws, **common, **grid)
    var = pd.DataFrame(sw["rows"]["variant"], columns=VARIANT)
    for c in ("lookback", "skip", "rebalance"):
        var[c] = var[c].astype(int)
    k = len(var)
    null_single, null_max = np.array(sw["rows"]["null_single"]).ravel(), np.array(sw["rows"]["null_max"]).ravel()
    best = var.set_index("variant").loc[sw["best"]]
    family = var.groupby("signal").agg(
        **{
            "variants": ("variant", "size"),
            "mean gross Sharpe": ("gross Sharpe", "mean"),
            "best gross Sharpe": ("gross Sharpe", "max"),
            "mean net Sharpe": ("net Sharpe", "mean"),
            "best net Sharpe": ("net Sharpe", "max"),
            "mean turnover": ("turnover", "mean"),
        }
    )
    ladder = pd.DataFrame(
        {
            "question": [
                "Is the Sharpe ratio above zero? (normal approximation)",
                "Do random portfolios built the same way do as well?",
                f"Does the best of {k} random strategies do as well?",
            ],
            "p-value": [sw["best_naive_p"], sw["best_single_null_p"], sw["best_max_null_p"]],
        }
    )
    _check(
        checks,
        "engine",
        "costs never increase a Sharpe ratio",
        bool((var["net Sharpe"] <= var["gross Sharpe"] + 1e-12).all()),
        f"{k} variants",
    )
    mirror = var[(var["signal"] == "momentum") & (var["skip"] == 0)].merge(
        var[var["signal"] == "reversal"], on=["lookback", "rebalance"]
    )
    gap = float((mirror["gross Sharpe_x"] + mirror["gross Sharpe_y"]).abs().max()) if len(mirror) else 0.0
    _check(
        checks,
        "engine",
        "reversal is the exact mirror of momentum without skip",
        gap < 1e-6,
        f"{len(mirror)} pairs, largest sum of the two Sharpe ratios {gap:.1e}",
    )
    _check(
        checks,
        "null",
        "random portfolios have no edge",
        abs(null_single.mean()) < 4 * null_single.std() / np.sqrt(len(null_single)) + 0.03,
        f"mean Sharpe {null_single.mean():+.3f} over {len(null_single)} draws",
    )
    _check(
        checks,
        "null",
        "width of the null agrees with the analytic standard error",
        abs(null_single.std() / sw["best_std_error"] - 1) < 0.25,
        f"{null_single.std():.3f} simulated, {sw['best_std_error']:.3f} from the formula",
    )
    _check(
        checks,
        "null",
        "searching raises the bar",
        sw["best_max_null_p"] >= sw["best_single_null_p"] and null_max.mean() > null_single.mean() + null_single.std(),
        f"mean of the best of {k}: {null_max.mean():.2f}, against {null_single.mean():.2f} for one",
    )
    _check(
        checks,
        "null",
        "formula for the expected maximum agrees with simulation",
        abs(sw["expected_max_sharpe"] / sw["max_null_mean"] - 1) < 0.3,
        f"{sw['expected_max_sharpe']:.2f} against {sw['max_null_mean']:.2f}",
        warn=True,
    )

    # 2. the best variant in detail
    b = {
        "signal": best["signal"],
        "lookback": int(best["lookback"]),
        "skip": int(best["skip"]),
        "rebalance": int(best["rebalance"]),
    }
    series = out / "best.csv"
    det = engine.call(
        "run",
        cost_bps=cfg["cost_bps"],
        start=int(sw["start"]),
        draws=0,
        bootstrap=boot,
        block=cfg["block"],
        out=series,
        **common,
        **b,
    )
    curve = pd.read_csv(series)
    series.unlink()
    cost_rows = {}
    for bps in cfg["costs_bps"]:
        r = engine.call("run", cost_bps=bps, start=int(sw["start"]), draws=0, bootstrap=0, **common, **b)
        cost_rows[bps] = {
            "net return": r["net_return"],
            "net Sharpe": r["net_sharpe"],
            "net drawdown": r["net_max_drawdown"],
        }
    costs = pd.DataFrame(cost_rows).T
    costs.index.name = "cost, bp"
    annual_turnover = det["turnover"] * 252 / b["rebalance"]
    breakeven = det["gross_return"] / annual_turnover * 1e4 if annual_turnover > 0 else float("inf")
    _check(
        checks,
        "engine",
        "detailed run reproduces the sweep",
        abs(det["gross_sharpe"] - sw["best_gross_sharpe"]) < 1e-6
        and abs(det["net_sharpe"] - sw["best_net_sharpe"]) < 1e-6,
        f"gross {det['gross_sharpe']:.4f}, net {det['net_sharpe']:.4f}",
    )
    _check(
        checks,
        "engine",
        "net return falls linearly with the cost rate",
        abs(
            (costs["net return"].iloc[0] - costs["net return"].iloc[-1])
            / (annual_turnover * (cfg["costs_bps"][-1] - cfg["costs_bps"][0]) * 1e-4)
            - 1
        )
        < 0.02,
        f"annual turnover {annual_turnover:.0f} times capital, break-even cost {breakeven:.1f} bp",
    )
    _check(
        checks,
        "inference",
        "bootstrap interval contains the estimate",
        det["net_sharpe_lower"] < det["net_sharpe"] < det["net_sharpe_upper"],
        f"net Sharpe {det['net_sharpe']:.2f}, 95% interval {det['net_sharpe_lower']:.2f} to {det['net_sharpe_upper']:.2f}",
    )

    # 3. walk-forward selection
    wfc = cfg["walk_forward"]
    wf = engine.call("walkforward", cost_bps=cfg["cost_bps"], **common, **grid, **wfc)
    blocks = pd.DataFrame(
        wf["rows"]["block"], columns=["first day", "chosen variant", "Sharpe in training", "return in the block"]
    )
    blocks["first day"] = blocks["first day"].astype(int)
    _check(
        checks,
        "selection",
        "training Sharpe overstates what selection earns",
        wf["mean_trained_sharpe"] > wf["selected_sharpe"],
        f"{wf['mean_trained_sharpe']:.2f} in training, {wf['selected_sharpe']:.2f} out of sample",
    )
    _check(
        checks,
        "selection",
        "out-of-sample days are the same for every variant",
        int(wf["oos_days"]) == int(sw["days"]) - wfc["train"],
        f"{int(wf['oos_days'])} days",
    )

    plotting.overview(var, null_single, null_max, sw, curve, blocks, out / "figures" / "overview.png")

    val = pd.DataFrame(checks)
    counts = val["status"].value_counts().to_dict()
    top = var.sort_values("gross Sharpe", ascending=False).head(8).drop(columns=["signal"])
    L = [
        "# BackLab report",
        "",
        f"Data: {int(desc['assets'])} stocks, {int(desc['days'])} days. Equal-weight portfolio: return {desc['benchmark_return']:.1%} a year, volatility {desc['benchmark_volatility']:.1%}, Sharpe ratio {desc['benchmark_sharpe']:.2f}.",
        "",
        f"Strategies: long the top {cfg['fraction']:.0%} of a signal, short the bottom {cfg['fraction']:.0%}, equal weights, no net exposure. Costs: {cfg['cost_bps']} bp per unit traded. "
        f"All {k} variants trade the same {int(sw['days'])} days.",
        "",
        "## 1. The search",
        "",
        md_table(family, 2),
        "",
        "The eight best variants before costs:",
        "",
        md_table(top, 3, index=False),
        "",
        f"{int(sw['positive_variants'])} of {k} variants have a positive gross Sharpe ratio. Mean absolute correlation between the daily returns of two variants: {sw['mean_abs_correlation']:.2f}.",
        "",
        f"## 2. Is the best one real? `{sw['best']}`",
        "",
        f"Gross Sharpe ratio {sw['best_gross_sharpe']:.2f} (standard error {sw['best_std_error']:.2f}).",
        "",
        md_table(ladder, 4, index=False),
        "",
        f"Random portfolios: mean Sharpe {sw['single_null_mean']:+.2f}, 95th percentile {sw['single_null_95']:.2f}. Best of {k} random strategies: mean {sw['max_null_mean']:.2f}, 95th percentile {sw['max_null_95']:.2f}. "
        f"Analytic expected maximum of {k} independent trials: {sw['expected_max_sharpe']:.2f}; deflated Sharpe ratio {sw['deflated_sharpe']:.2f}.",
        "",
        "## 3. After costs",
        "",
        f"Turnover {det['turnover']:.2f} of capital per rebalance, {annual_turnover:.0f} times a year. Gross return {det['gross_return']:.1%} a year, so the strategy breaks even at {breakeven:.1f} bp.",
        "",
        md_table(costs, 3),
        "",
        f"At {cfg['cost_bps']} bp: net Sharpe ratio {det['net_sharpe']:.2f}, bootstrap 95% interval {det['net_sharpe_lower']:.2f} to {det['net_sharpe_upper']:.2f}, {det['net_share_not_positive']:.0%} of resamples at or below zero. "
        f"Largest drawdown {det['net_max_drawdown']:.1%} of capital, longest time below a previous peak {int(det['net_longest_underwater'])} days.",
        "",
        "## 4. Choosing without hindsight",
        "",
        f"Every {wfc['step']} days the variant with the best net Sharpe ratio over the previous {wfc['train']} days is selected and traded for the next {wfc['step']}.",
        "",
        md_table(blocks, 3, index=False),
        "",
        f"Mean Sharpe ratio of the chosen variant in its training window: {wf['mean_trained_sharpe']:.2f}. Sharpe ratio actually earned over the {int(wf['oos_days'])} out-of-sample days: {wf['selected_sharpe']:.2f} (standard error {wf['selected_std_error']:.2f}). "
        f"Over the same days the variants average {wf['mean_variant_sharpe']:.2f}, the best in hindsight reaches {wf['best_variant_sharpe']:.2f}, and the selection beats {int(wf['variants_beaten'])} of {int(wf['variants'])}. "
        f"The full-sample winner, `{wf['hindsight_best']}`, has a net Sharpe ratio of {wf['hindsight_full_sharpe']:.2f} over the whole period and {wf['hindsight_oos_sharpe']:.2f} over the out-of-sample days.",
        "",
        "## Validation",
        "",
        f"{counts.get('PASS', 0)} PASS, {counts.get('WARN', 0)} WARN, {counts.get('FAIL', 0)} FAIL. See `validation.md`.",
        "",
    ]
    (out / "report.md").write_text("\n".join(L))
    (out / "validation.md").write_text("# Validation checks\n\n" + md_table(val, index=False) + "\n")
    metrics = {
        "benchmark": {x: v for x, v in desc.items() if x != "rows"},
        "variants": var.set_index("variant").to_dict("index"),
        "search": {x: v for x, v in sw.items() if x != "rows"},
        "best": {x: v for x, v in det.items() if x != "rows"},
        "costs": {str(x): v for x, v in costs.to_dict("index").items()},
        "breakeven_bp": breakeven,
        "walk_forward": {x: v for x, v in wf.items() if x != "rows"},
        "checks": checks,
    }
    (out / "metrics.json").write_text(json.dumps(metrics, indent=1, default=float))
    return metrics
