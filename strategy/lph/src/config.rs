//! Configuration types for LPH Monitor.

use alloy::primitives::Address;

/// Configuration for LPHStrategy (parameters only; clients are passed to `LPHStrategy::new`).
pub struct LPHStrategyConfig {
    /// Ethereum address that owns the Uniswap V3 LP positions
    pub owner: Address,
    /// Binance futures symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Ethereum address of the BASE token (e.g., BNB, ETH)
    pub base_token_address: Address,
    /// Ethereum address of the USDT token
    pub usdt_token_address: Address,
    /// Threshold for base_delta_ratio (n): execute only when base_delta_ratio > n
    pub base_delta_ratio_threshold: f64,
    /// Threshold for base_delta magnitude (m): execute only when |base_delta| > m; also used as quantity step for rounding
    pub base_delta_threshold: f64,
}
