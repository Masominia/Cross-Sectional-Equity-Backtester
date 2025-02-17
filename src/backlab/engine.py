"""Python side of the Rust engine: unpacks the price panel, runs the binary and parses its output."""

from __future__ import annotations

import gzip
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "engine" / "target" / "release" / "backlab"
RAW = ROOT / "data" / "raw" / "prices_300_stocks.csv.gz"
PRICES = ROOT / "data" / "processed" / "prices.csv"


def available() -> bool:
    return BINARY.exists()


def prices() -> Path:
    """Plain-text copy of the panel for the Rust reader, created on first use."""
    if not PRICES.exists():
        PRICES.parent.mkdir(parents=True, exist_ok=True)
        with gzip.open(RAW, "rb") as src, open(PRICES, "wb") as dst:
            shutil.copyfileobj(src, dst)
    return PRICES


def _value(text: str):
    try:
        return float(text)
    except ValueError:
        return text


def call(command: str, path: Path | None = None, **params) -> dict:
    """`call("run", signal="reversal", lookback=2)` -> {"gross_sharpe": ..., "rows": {...}}"""
    args = [f"{k}={','.join(map(str, v)) if isinstance(v, (list, tuple)) else v}" for k, v in params.items()]
    out = subprocess.run(
        [str(BINARY), command, str(path or prices()), *args], capture_output=True, text=True, check=True
    ).stdout
    res: dict = {"rows": {}}
    for line in out.splitlines():
        f = line.split()
        if f[0] == "row":
            res["rows"].setdefault(f[1], []).append([_value(x) for x in f[2:]])
        else:
            res[f[0]] = _value(f[1])
    return res
