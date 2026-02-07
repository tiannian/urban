//! Shared types for LP Hedging strategy.

use serde::{Deserialize, Serialize};

/// Monitoring snapshot containing all computed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    /// Blockchain block number at which the on-chain LP position data was read
    pub block_number: u64,
    /// Futures symbol
    pub symbol: String,
    /// Amount of BASE tokens in LP position
    pub amm_base_amount: f64,
    /// Amount of USDT tokens in LP position
    pub amm_usdt_amount: f64,
    /// Amount of BASE that can be collected as fees from the LP position
    pub amm_collectable_base: f64,
    /// Amount of USDT that can be collected as fees from the LP position
    pub amm_collectable_usdt: f64,
    /// Total value in USDT of collectable AMM fees (score)
    pub amm_collectable_value_usdt: f64,
    /// Net futures position in BASE units (positive = long, negative = short)
    pub futures_position: f64,
    /// Unrealized PnL of the futures position in USDT
    pub unrealized_pnl: f64,
    /// Timestamp from Binance position data (in milliseconds since Unix epoch)
    pub futures_timestamp: i64,
    /// Current BASE price in USDT
    pub base_price_usdt: f64,
    /// Net BASE exposure (amm_base_amount + futures_position)
    pub base_delta: f64,
    /// Relative deviation ratio
    pub base_delta_ratio: f64,
    /// Total AMM position value in USDT
    pub amm_total_value_usdt: f64,
    /// Total combined value in USDT (AMM value plus unrealized PnL)
    pub total_value_usdt: f64,
}
