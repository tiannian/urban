//! LP Hedging strategy crate.
//!
//! Provides monitoring for LP hedging setups that combine CEX futures
//! with on-chain AMM positions.

mod monitor;

pub use monitor::{LPHMonitor, LPHMonitorConfig, MonitoringSnapshot};
