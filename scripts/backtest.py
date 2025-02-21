"""Backtest one strategy on the bundled panel or on your own price file.

python scripts/backtest.py signal=reversal lookback=2 rebalance=5 cost_bps=10
python scripts/backtest.py prices=my_prices.csv signal=momentum lookback=120 skip=20
"""

import sys

from backlab import engine

if __name__ == "__main__":
    params = dict(a.split("=", 1) for a in sys.argv[1:])
    path = params.pop("prices", None)
    r = engine.call("run", path, **params)
    print(f"{r['signal']}: {int(r['days'])} days, turnover {r['turnover']:.2f} per rebalance")
    for side in ("gross", "net"):
        ret, sr, dd = r[side + "_return"], r[side + "_sharpe"], r[side + "_max_drawdown"]
        print(f"{side:<6} return {ret:>7.2%}  Sharpe {sr:>5.2f}  drawdown {dd:>6.1%}")
    if "null_p_value" in r:
        print(f"random portfolios do as well with probability {r['null_p_value']:.3f}")
    if "net_sharpe_lower" in r:
        print(f"net Sharpe ratio, 95% interval: {r['net_sharpe_lower']:.2f} to {r['net_sharpe_upper']:.2f}")
