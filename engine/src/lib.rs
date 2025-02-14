//! A cross-sectional backtester: signals rank the stocks, portfolios follow the ranks, and the
//! inference modules ask how much of the resulting Sharpe ratio random portfolios would have earned.
//!
//! Timing convention used everywhere: a signal computed from prices up to the close of day t
//! sets the weights at that close; the first return it earns is that of day t + 1.

pub mod data;
pub mod engine;
pub mod inference;
pub mod math;
pub mod metrics;
pub mod portfolio;
pub mod signal;

pub use data::panel::Panel;
pub use engine::backtest::{run, Config, Mode, Outcome};
pub use signal::Signal;
