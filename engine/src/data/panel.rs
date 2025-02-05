//! Prices of N assets over T days, stored day by day, with daily simple returns alongside.

#[derive(Debug)]
pub struct Panel {
    pub names: Vec<String>,
    pub days: usize,
    pub assets: usize,
    prices: Vec<f64>,
    returns: Vec<f64>,
}

impl Panel {
    /// `prices` holds `days` rows of `names.len()` values. All prices must be positive.
    pub fn new(names: Vec<String>, prices: Vec<f64>) -> Result<Panel, String> {
        let assets = names.len();
        if assets == 0 || prices.is_empty() || prices.len() % assets != 0 {
            return Err("price table does not match the number of names".into());
        }
        if prices.iter().any(|p| !(*p > 0.0) || !p.is_finite()) {
            return Err("prices must be positive and finite".into());
        }
        let days = prices.len() / assets;
        let mut returns = vec![0.0; prices.len()];
        for t in 1..days {
            for i in 0..assets {
                returns[t * assets + i] = prices[t * assets + i] / prices[(t - 1) * assets + i] - 1.0;
            }
        }
        Ok(Panel { names, days, assets, prices, returns })
    }

    pub fn price(&self, day: usize, asset: usize) -> f64 {
        self.prices[day * self.assets + asset]
    }

    pub fn prices_on(&self, day: usize) -> &[f64] {
        &self.prices[day * self.assets..(day + 1) * self.assets]
    }

    /// Returns from the close of `day - 1` to the close of `day`; zero on day 0.
    pub fn returns_on(&self, day: usize) -> &[f64] {
        &self.returns[day * self.assets..(day + 1) * self.assets]
    }

    /// The first `assets` columns only.
    pub fn truncated(&self, assets: usize) -> Panel {
        let n = assets.min(self.assets);
        let mut prices = Vec::with_capacity(self.days * n);
        for t in 0..self.days {
            prices.extend_from_slice(&self.prices_on(t)[..n]);
        }
        Panel::new(self.names[..n].to_vec(), prices).expect("a valid panel stays valid")
    }

    /// Daily return of the portfolio that holds every asset equally and rebalances every day.
    pub fn equal_weight_returns(&self) -> Vec<f64> {
        (0..self.days).map(|t| self.returns_on(t).iter().sum::<f64>() / self.assets as f64).collect()
    }
}
