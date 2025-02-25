# BackLab report

Data: 300 stocks, 1362 days. Equal-weight portfolio: return 12.7% a year, volatility 24.4%, Sharpe ratio 0.52.

Strategies: long the top 20% of a signal, short the bottom 20%, equal weights, no net exposure. Costs: 10 bp per unit traded. All 34 variants trade the same 1091 days.

## 1. The search

| signal | variants | mean gross Sharpe | best gross Sharpe | mean net Sharpe | best net Sharpe | mean turnover |
|---|---|---|---|---|---|---|
| lowvol | 8 | -0.35 | -0.15 | -0.42 | -0.25 | 0.35 |
| momentum | 16 | -0.08 | 0.21 | -0.25 | 0.10 | 0.71 |
| reversal | 10 | 0.50 | 1.43 | 0.07 | 0.67 | 1.45 |

The eight best variants before costs:

| variant | lookback | skip | rebalance | gross Sharpe | net Sharpe | turnover | gross return | net return | net drawdown | p against random |
|---|---|---|---|---|---|---|---|---|---|---|
| reversal_2_0_5 | 2 | 0 | 5 | 1.427 | 0.674 | 1.552 | 0.149 | 0.070 | 0.210 | 3.99e-03 |
| reversal_1_0_5 | 1 | 0 | 5 | 1.143 | 0.352 | 1.554 | 0.114 | 0.035 | 0.246 | 0.012 |
| reversal_5_0_5 | 5 | 0 | 5 | 0.844 | 0.107 | 1.571 | 0.091 | 0.011 | 0.187 | 0.036 |
| reversal_10_0_5 | 10 | 0 | 5 | 0.564 | 0.039 | 1.116 | 0.061 | 4.24e-03 | 0.167 | 0.118 |
| reversal_2_0_20 | 2 | 0 | 20 | 0.413 | 0.210 | 1.566 | 0.040 | 0.021 | 0.192 | 0.208 |
| reversal_1_0_20 | 1 | 0 | 20 | 0.349 | 0.139 | 1.567 | 0.033 | 0.013 | 0.170 | 0.238 |
| reversal_5_0_20 | 5 | 0 | 20 | 0.326 | 0.108 | 1.579 | 0.030 | 9.92e-03 | 0.163 | 0.271 |
| reversal_20_0_5 | 20 | 0 | 5 | 0.225 | -0.141 | 0.798 | 0.025 | -0.016 | 0.203 | 0.331 |

16 of 34 variants have a positive gross Sharpe ratio. Mean absolute correlation between the daily returns of two variants: 0.41.

## 2. Is the best one real? `reversal_2_0_5`

Gross Sharpe ratio 1.43 (standard error 0.45).

| question | p-value |
|---|---|
| Is the Sharpe ratio above zero? (normal approximation) | 8.32e-04 |
| Do random portfolios built the same way do as well? | 3.99e-03 |
| Does the best of 34 random strategies do as well? | 0.0399 |

Random portfolios: mean Sharpe -0.00, 95th percentile 0.75. Best of 34 random strategies: mean 0.99, 95th percentile 1.37. Analytic expected maximum of 34 independent trials: 0.99; deflated Sharpe ratio 0.83.

## 3. After costs

Turnover 1.55 of capital per rebalance, 78 times a year. Gross return 14.9% a year, so the strategy breaks even at 19.0 bp.

| cost, bp | net return | net Sharpe | net drawdown |
|---|---|---|---|
| 0 | 0.149 | 1.427 | 0.113 |
| 2 | 0.133 | 1.277 | 0.128 |
| 5 | 0.110 | 1.051 | 0.151 |
| 10 | 0.070 | 0.674 | 0.210 |
| 20 | -7.98e-03 | -0.076 | 0.398 |
| 30 | -0.086 | -0.802 | 0.594 |

At 10 bp: net Sharpe ratio 0.67, bootstrap 95% interval -0.59 to 1.84, 15% of resamples at or below zero. Largest drawdown 21.0% of capital, longest time below a previous peak 639 days.

## 4. Choosing without hindsight

Every 63 days the variant with the best net Sharpe ratio over the previous 378 days is selected and traded for the next 63.

| first day | chosen variant | Sharpe in training | return in the block |
|---|---|---|---|
| 649 | reversal_2_0_5 | 2.480 | 0.013 |
| 712 | reversal_2_0_5 | 2.354 | -0.012 |
| 775 | reversal_2_0_5 | 2.128 | -0.026 |
| 838 | reversal_2_0_5 | 1.938 | -0.030 |
| 901 | reversal_2_0_5 | 1.811 | -0.046 |
| 964 | momentum_120_20_5 | 0.449 | 0.016 |
| 1,027 | momentum_60_0_20 | 0.825 | 0.039 |
| 1,090 | momentum_120_20_5 | 1.192 | -0.028 |
| 1,153 | lowvol_20_0_20 | 1.364 | 0.037 |
| 1,216 | momentum_250_0_20 | 1.406 | -0.131 |
| 1,279 | lowvol_120_0_5 | 0.779 | 9.89e-03 |
| 1,342 | momentum_250_0_20 | 0.749 | -0.012 |

Mean Sharpe ratio of the chosen variant in its training window: 1.46. Sharpe ratio actually earned over the 713 out-of-sample days: -0.51 (standard error 0.59). Over the same days the variants average -0.10, the best in hindsight reaches 0.54, and the selection beats 3 of 34. The full-sample winner, `reversal_2_0_5`, has a net Sharpe ratio of 0.67 over the whole period and -0.48 over the out-of-sample days.

## Validation

11 PASS, 0 WARN, 0 FAIL. See `validation.md`.
