//! Configuration types for Uniswap V3 clients.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

/// Configuration for UniswapV3PositionManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV3PositionManagerConfig {
    /// The contract address of the Uniswap V3 PositionManager contract
    pub address: Address,
}
