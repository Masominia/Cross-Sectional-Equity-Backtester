"""Static figures for the report (matplotlib, PNG for the README)."""

from __future__ import annotations

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

# Validated categorical order (fixed, never cycled) + recessive ink.
SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
NEUTRAL = "#8a8985"
CRITICAL = "#c4271a"

plt.rcParams.update(
    {
        "figure.facecolor": SURFACE,
        "axes.facecolor": SURFACE,
        "savefig.facecolor": SURFACE,
        "axes.edgecolor": GRID,
        "axes.labelcolor": INK2,
        "xtick.color": INK2,
        "ytick.color": INK2,
        "text.color": INK,
        "axes.grid": True,
        "grid.color": GRID,
        "grid.linewidth": 0.6,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "font.size": 9,
        "axes.titlesize": 10,
        "axes.titleweight": "bold",
        "legend.frameon": False,
        "lines.linewidth": 1.6,
        "figure.dpi": 150,
        "axes.axisbelow": True,
    }
)


def _save(fig, path):
    fig.tight_layout()
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)


def overview(var, null_single, null_max, sw, curve, blocks, path):
    fig, ax = plt.subplots(2, 2, figsize=(11, 7.6))
    a = ax[0, 0]
    for i, (name, g) in enumerate(var.groupby("signal")):
        a.plot(g["gross Sharpe"], g["net Sharpe"], "o", color=SERIES[i], ms=5, label=name)
    lim = [var[["gross Sharpe", "net Sharpe"]].min().min() - 0.1, var["gross Sharpe"].max() + 0.1]
    a.plot(lim, lim, color=NEUTRAL, lw=1, ls="--", label="no cost")
    a.axhline(0, color=GRID, lw=1)
    a.set(title="Every variant of the search", xlabel="Sharpe ratio before costs", ylabel="after costs")
    a.legend()

    a = ax[0, 1]
    bins = np.linspace(min(null_single.min(), -1.5), max(null_max.max(), sw["best_gross_sharpe"]) + 0.2, 45)
    a.hist(null_single, bins=bins, color=SERIES[0], alpha=0.75, label="one random strategy")
    a.hist(null_max, bins=bins, color=SERIES[1], alpha=0.75, label=f"best of {len(var)} random strategies")
    a.axvline(sw["best_gross_sharpe"], color=INK, lw=1.5, label="best variant found")
    a.set(title="What luck alone produces", xlabel="gross Sharpe ratio", ylabel="draws")
    a.legend(fontsize=7)

    a = ax[1, 0]
    a.plot(curve["day"], 100 * curve["gross"], color=SERIES[0], label="before costs")
    a.plot(curve["day"], 100 * curve["net"], color=SERIES[1], label="after costs")
    a.axhline(0, color=GRID, lw=1)
    a.set(title=f"Best variant ({sw['best']}): cumulative profit", xlabel="day", ylabel="% of capital")
    a.legend()

    a = ax[1, 1]
    a.plot(blocks["Sharpe in training"], 100 * blocks["return in the block"], "o", color=SERIES[0], ms=7)
    a.axhline(0, color=INK2, lw=0.8)
    a.axvline(0, color=INK2, lw=0.8)
    a.set(
        title="Walk-forward: what was promised, what was earned",
        xlabel="Sharpe ratio of the chosen variant in training",
        ylabel="its return in the next block, % of capital",
    )
    _save(fig, path)
