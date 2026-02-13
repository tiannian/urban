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
}
