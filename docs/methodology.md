# Methods

## Timing

Prices are daily closes. A signal for day t uses prices up to and including day t. Weights are set at that close and the first return they earn is that of day t + 1. Every signal declares a warm-up, the number of earlier days it needs, and the backtest refuses to start inside it. All variants of a search start on the largest warm-up of the grid, so they cover the same days.

## Signals

- Momentum: return over `lookback` days ending `skip` days ago.
- Reversal: minus the return over the last `lookback` days. With no skip it is the exact negative of momentum, which the pipeline checks.
- Low volatility: minus the standard deviation of daily returns over `lookback` days.
- Combination: mean of the cross-sectional z-scores of its parts.

## Portfolios and costs

Stocks are sorted by score. Long-short: the top fraction gets +0.5/k each and the bottom fraction -0.5/k, so gross exposure is one and net exposure zero. Long-only: the top fraction gets 1/k each and the return is reported in excess of the equal-weight portfolio of all stocks.

Between rebalances the positions drift with prices. At a rebalance, turnover is the sum of absolute differences between drifted and target weights, and the cost is turnover times the rate in basis points, charged to the next day's return. Reported turnover leaves out the first trade from cash.

## Sharpe ratio and its error

Annualised with 252 days, no risk-free rate since the series are self-financing. The standard error is `sqrt((1 - g3 s + (g4 - 1)/4 s^2) / (n - 1))` in daily units, with s the daily Sharpe ratio, g3 the skewness and g4 the kurtosis of the daily returns.

## Random-portfolio null

The backtest is rerun with the scores replaced by uniform random numbers, drawn afresh at every rebalance. Universe, days, number of names per leg, rebalancing dates and position drift are unchanged. The p-value of an observed Sharpe ratio is `(1 + number of draws at or above it) / (1 + draws)`.

For a search over N variants, each draw runs one random strategy per variant, with that variant's rebalancing interval, and keeps the largest Sharpe ratio. The observed winner is compared with this distribution.

## Expected maximum and deflated Sharpe ratio

For N independent trials whose Sharpe ratios have variance V, the expected maximum under no skill is approximately `sqrt(V) ((1 - c) z(1 - 1/N) + c z(1 - 1/(N e)))`, with z the normal quantile and c Euler's constant. The deflated Sharpe ratio is the probability, under the normal approximation with the standard error above, that the true Sharpe ratio of the winner exceeds this value.

## Bootstrap

Stationary bootstrap: a resampled series is built by starting at a random day and, at each step, either moving to the next day or, with probability one over the mean block length, jumping to a random day. The interval is the percentile interval of the Sharpe ratios of the resamples.

## Walk-forward

All variants are run once over the full period, after costs. At the start of each block the variant with the highest Sharpe ratio over the training window is chosen and its returns over the block are taken. The comparison set is every variant over the same out-of-sample days.
