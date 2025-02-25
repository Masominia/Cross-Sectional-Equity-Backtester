# Validation checks

| area | check | status | detail |
|---|---|---|---|
| engine | costs never increase a Sharpe ratio | PASS | 34 variants |
| engine | reversal is the exact mirror of momentum without skip | PASS | 2 pairs, largest sum of the two Sharpe ratios 0.0e+00 |
| null | random portfolios have no edge | PASS | mean Sharpe -0.004 over 500 draws |
| null | width of the null agrees with the analytic standard error | PASS | 0.481 simulated, 0.454 from the formula |
| null | searching raises the bar | PASS | mean of the best of 34: 0.99, against -0.00 for one |
| null | formula for the expected maximum agrees with simulation | PASS | 0.99 against 0.99 |
| engine | detailed run reproduces the sweep | PASS | gross 1.4268, net 0.6737 |
| engine | net return falls linearly with the cost rate | PASS | annual turnover 78 times capital, break-even cost 19.0 bp |
| inference | bootstrap interval contains the estimate | PASS | net Sharpe 0.67, 95% interval -0.59 to 1.84 |
| selection | training Sharpe overstates what selection earns | PASS | 1.46 in training, -0.51 out of sample |
| selection | out-of-sample days are the same for every variant | PASS | 713 days |
