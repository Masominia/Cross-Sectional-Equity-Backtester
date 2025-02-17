import numpy as np
import pandas as pd
import pytest

from backlab import engine

pytestmark = pytest.mark.skipif(not engine.available(), reason="Rust engine not built")


def _panel(tmp_path, days=500, assets=30, drift=0.0, seed=0):
    rng = np.random.default_rng(seed)
    mu = drift * (np.arange(assets) / (assets - 1) - 0.5)
    prices = 100 * np.cumprod(1 + mu + 0.02 * rng.standard_normal((days, assets)), axis=0)
    path = tmp_path / "prices.csv"
    pd.DataFrame(prices, columns=[f"S{i}" for i in range(assets)]).rename_axis("day").to_csv(path)
    return path, prices


def test_benchmark_matches_numpy(tmp_path):
    path, prices = _panel(tmp_path)
    r = (prices[1:] / prices[:-1] - 1).mean(axis=1)
    d = engine.call("describe", path)
    assert d["days"] == 500 and d["assets"] == 30
    assert abs(d["benchmark_return"] - r.mean() * 252) < 1e-9
    assert abs(d["benchmark_sharpe"] - r.mean() / r.std(ddof=1) * np.sqrt(252)) < 1e-8


def test_momentum_matches_an_independent_backtest(tmp_path):
    path, prices = _panel(tmp_path, days=300, assets=20)
    look, start, k = 30, 30, 4
    ret = prices[1:] / prices[:-1] - 1  # ret[t - 1] is the return of day t
    daily = []
    for t in range(start, 299):  # rebalanced every day, so no drift to model
        score = prices[t] / prices[t - look] - 1
        order = np.argsort(score, kind="stable")
        w = np.zeros(20)
        w[order[:k]], w[order[-k:]] = -0.5 / k, 0.5 / k
        daily.append(w @ ret[t])
    r = engine.call(
        "run", path, signal="momentum", lookback=look, skip=0, rebalance=1, cost_bps=0, draws=0, bootstrap=0
    )
    assert r["days"] == len(daily)
    assert abs(r["gross_return"] - np.mean(daily) * 252) < 1e-9
    assert abs(r["gross_sharpe"] - np.mean(daily) / np.std(daily, ddof=1) * np.sqrt(252)) < 1e-7


def test_edge_is_detected_and_noise_is_not(tmp_path):
    trend, _ = _panel(tmp_path, days=700, drift=0.004, seed=1)
    r = engine.call("run", trend, signal="momentum", lookback=120, skip=0, draws=200, bootstrap=200)
    assert r["gross_sharpe"] > 1.0 and r["null_p_value"] < 0.03 and r["net_sharpe"] < r["gross_sharpe"]
    assert r["net_sharpe_lower"] < r["net_sharpe"] < r["net_sharpe_upper"]
    noise, _ = _panel(tmp_path, days=700, drift=0.0, seed=2)
    assert (
        engine.call("run", noise, signal="momentum", lookback=120, skip=0, draws=200, bootstrap=0)["null_p_value"]
        > 0.02
    )


def test_sweep_and_walk_forward(tmp_path):
    path, _ = _panel(tmp_path, days=600)
    grid = {"momentum": [20, 60], "skips": [0], "reversal": [5], "lowvol": [20], "rebalances": [10]}
    s = engine.call("sweep", path, draws=40, **grid)
    assert s["variants"] == 4 and len(s["rows"]["variant"]) == 4 and len(s["rows"]["null_max"]) == 40
    assert s["best_max_null_p"] >= s["best_single_null_p"] - 1e-12
    w = engine.call("walkforward", path, train=200, step=50, **grid)
    assert w["oos_days"] == s["days"] - 200 and w["blocks"] == len(w["rows"]["block"])


def test_bad_input_is_reported(tmp_path):
    bad = tmp_path / "bad.csv"
    bad.write_text("day,A,B\n0,1\n")
    with pytest.raises(Exception):  # noqa: B017
        engine.call("describe", bad)
