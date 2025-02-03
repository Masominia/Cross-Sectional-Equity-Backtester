# Data

`raw/prices_300_stocks.csv.gz`: adjusted daily closing prices of 300 Nasdaq-listed stocks over 1,362 consecutive
trading days (about five and a half years), one row per day, one column per ticker. Calendar dates are replaced by
a day counter. Source: the Nasdaq panel of the `skfolio/skfolio-datasets` repository on GitHub, originally from
Yahoo Finance.

Selection: complete history, no daily move above 60%, fewer than 5% unchanged closes, then 300 tickers drawn at random.

`processed/prices.csv` is the same file unpacked for the Rust reader. It is created on first use and not kept in the repository.

Limits: only stocks listed for the whole period (survivorship bias); closing prices only, so no bid-ask spread,
volume or borrow cost; short and dominated by one market crash.
