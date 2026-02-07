//! LP Hedging strategy crate.
//!
//! Provides monitoring for LP hedging setups that combine CEX futures
//! with on-chain AMM positions.

pub mod config;
mod monitor;
mod types;

pub use config::LPHMonitorConfig;
pub use monitor::LPHMonitor;
pub use types::MonitoringSnapshot;
