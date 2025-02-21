"""Run the default search and print the significance of its winner.

python scripts/search.py [draws]
"""

import sys

from backlab import engine

if __name__ == "__main__":
    s = engine.call("sweep", draws=int(sys.argv[1]) if len(sys.argv) > 1 else 200)
    print(f"{int(s['variants'])} variants, best: {s['best']} with a gross Sharpe ratio of {s['best_gross_sharpe']:.2f}")
    print(f"p-value, taken alone                 {s['best_naive_p']:.4f}")
    print(f"p-value against random portfolios    {s['best_single_null_p']:.4f}")
    print(f"p-value against the best of a search {s['best_max_null_p']:.4f}")
    print(f"net Sharpe ratio after costs         {s['best_net_sharpe']:.2f}")
