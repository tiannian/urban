//! LP Hedging Monitor implementation
//!
//! This module provides monitoring functionality for LP hedging setups that combine
//! centralized exchange (CEX) futures accounts with on-chain AMM positions.

use alloy::primitives::{Address, U256};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use binance::BinancePerpsClient;
use uniswapv3::UniswapV3PositionManager;

/// Configuration for LPHMonitor
pub struct LPHMonitorConfig {
    /// Uniswap V3 client instance
    pub uniswap_client: UniswapV3PositionManager,
    /// Binance futures client instance
    pub binance_client: BinancePerpsClient,
    /// Ethereum address that owns the Uniswap V3 LP positions
    pub owner: Address,
    /// Binance futures symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Ethereum address of the BASE token (e.g., BNB, ETH)
    pub base_token_address: Address,
    /// Ethereum address of the USDT token
    pub usdt_token_address: Address,
}

/// Monitoring snapshot containing all computed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    /// Current Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Blockchain block number at which the on-chain LP position data was read
    pub block_number: u64,
    /// Futures symbol
    pub symbol: String,
    /// Amount of BASE tokens in LP position
    pub amm_base_amount: f64,
    /// Amount of USDT tokens in LP position
    pub amm_usdt_amount: f64,
    /// Net futures position in BASE units (positive = long, negative = short)
    pub futures_position: f64,
    /// USDT balance on futures account
    pub futures_balance_usdt: f64,
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
    /// Total combined value in USDT
    pub total_value_usdt: f64,
}

/// LP Hedging Monitor
///
/// Monitors the overall account state for an LP hedging setup that combines:
/// - A centralized exchange (CEX) futures account
/// - An on-chain AMM position
pub struct LPHMonitor {
    /// Uniswap V3 client instance
    uniswap_client: UniswapV3PositionManager,
    /// Binance futures client instance
    binance_client: BinancePerpsClient,
    /// Ethereum address that owns the Uniswap V3 LP positions
    owner: Address,
    /// Binance futures symbol
    symbol: String,
    /// Ethereum address of the BASE token
    base_token_address: Address,
    /// Ethereum address of the USDT token
    usdt_token_address: Address,
}

impl LPHMonitor {
    /// Creates a new `LPHMonitor` instance
    ///
    /// # Arguments
    /// * `config` - A `LPHMonitorConfig` instance containing all configuration parameters and client instances
    ///
    /// # Returns
    /// A new `LPHMonitor` instance with both clients and configuration parameters configured
    pub fn new(config: LPHMonitorConfig) -> Self {
        Self {
            uniswap_client: config.uniswap_client,
            binance_client: config.binance_client,
            owner: config.owner,
            symbol: config.symbol,
            base_token_address: config.base_token_address,
            usdt_token_address: config.usdt_token_address,
        }
    }

    /// Performs a complete monitoring cycle by reading data from both clients and computing monitoring metrics
    ///
    /// # Returns
    /// A `MonitoringSnapshot` structure containing all monitoring metrics, or an error if data reading or computation fails
    pub async fn status(&mut self) -> Result<MonitoringSnapshot, Box<dyn std::error::Error>> {
        // Step 1: Read AMM LP Position Data
        self.uniswap_client.sync_lp(self.owner).await?;

        // Find the position matching base_token_address and usdt_token_address
        let position_data = self
            .uniswap_client
            .positions()
            .values()
            .find(|pos| {
                (pos.token0 == self.base_token_address && pos.token1 == self.usdt_token_address)
                    || (pos.token0 == self.usdt_token_address
                        && pos.token1 == self.base_token_address)
            })
            .ok_or_else(|| {
                anyhow!(
                    "No matching Uniswap position found for base_token={:?} and usdt_token={:?}",
                    self.base_token_address,
                    self.usdt_token_address
                )
            })?;

        // Determine which token is BASE and which is USDT
        let (amm_base_amount_raw, amm_usdt_amount_raw) =
            if position_data.token0 == self.base_token_address {
                (
                    position_data.withdrawable_amount0,
                    position_data.withdrawable_amount1,
                )
            } else {
                (
                    position_data.withdrawable_amount1,
                    position_data.withdrawable_amount0,
                )
            };

        // Convert U256 amounts to f64 (assuming 18 decimals for BASE and 6 decimals for USDT)
        // Note: In production, you should fetch actual token decimals from the contract
        const BASE_DECIMALS: u32 = 18;
        const USDT_DECIMALS: u32 = 6;

        let amm_base_amount = u256_to_f64(amm_base_amount_raw, BASE_DECIMALS);
        let amm_usdt_amount = u256_to_f64(amm_usdt_amount_raw, USDT_DECIMALS);

        // Get current block number
        let block_number = self.uniswap_client.get_block_number().await?;

        // Step 2: Read Binance Futures Position Data
        let positions = self.binance_client.get_position(&self.symbol).await?;

        let binance_position = positions
            .iter()
            .find(|p| p.symbol == self.symbol)
            .ok_or_else(|| {
                anyhow!(
                    "No matching Binance position found for symbol={}",
                    self.symbol
                )
            })?;

        // Extract futures position (convert from string to f64, preserving sign)
        let futures_position = binance_position
            .position_amt
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse position_amt: {}", e))?;

        // Extract futures balance (use isolated_wallet or isolated_margin)
        let futures_balance_usdt = binance_position
            .isolated_wallet
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse isolated_wallet: {}", e))?;

        // Extract base price
        let base_price_usdt = binance_position
            .mark_price
            .parse::<f64>()
            .map_err(|e| anyhow!("Failed to parse mark_price: {}", e))?;

        // Extract timestamp
        let futures_timestamp = binance_position.update_time;

        // Step 3: Compute Monitoring Metrics
        let base_delta = amm_base_amount + futures_position;

        // Compute base_reference with epsilon to avoid division by zero
        const EPSILON: f64 = 1e-8;
        let base_reference = amm_base_amount
            .abs()
            .max(futures_position.abs())
            .max(EPSILON);

        let base_delta_ratio = base_delta / base_reference;

        let amm_base_value_usdt = amm_base_amount * base_price_usdt;
        let amm_total_value_usdt = amm_base_value_usdt + amm_usdt_amount;
        let total_value_usdt = amm_total_value_usdt + futures_balance_usdt;

        // Step 4: Build and Return Monitoring Snapshot
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Failed to get timestamp: {}", e))?
            .as_millis() as i64;

        Ok(MonitoringSnapshot {
            timestamp,
            block_number,
            symbol: self.symbol.clone(),
            amm_base_amount,
            amm_usdt_amount,
            futures_position,
            futures_balance_usdt,
            futures_timestamp,
            base_price_usdt,
            base_delta,
            base_delta_ratio,
            amm_total_value_usdt,
            total_value_usdt,
        })
    }
}

/// Converts a U256 value to f64, accounting for token decimals
fn u256_to_f64(value: U256, decimals: u32) -> f64 {
    // Convert U256 to u128 (assuming it fits)
    // For values larger than u128::MAX, this will truncate, but that's acceptable for f64 precision
    let value_u128 = value.to::<u128>();

    // Divide by 10^decimals to get the decimal representation
    let divisor = 10_u128.pow(decimals);
    let whole_part = value_u128 / divisor;
    let fractional_part = value_u128 % divisor;

    // Combine whole and fractional parts
    // Use f64 arithmetic to preserve precision
    whole_part as f64 + (fractional_part as f64 / divisor as f64)
}
