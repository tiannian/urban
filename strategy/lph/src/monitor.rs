//! LP Hedging Monitor implementation
//!
//! This module provides monitoring functionality for LP hedging setups that combine
//! centralized exchange (CEX) futures accounts with on-chain AMM positions.

use alloy::primitives::{Address, U256};
use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use clients_binance::BinancePerpsClient;
use clients_uniswapv3::UniswapV3PositionManager;

use crate::config::LPHMonitorConfig;
use crate::types::MonitoringSnapshot;

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
    /// * `config` - A `LPHMonitorConfig` instance containing configuration parameters
    /// * `uniswap_client` - Uniswap V3 client instance
    /// * `binance_client` - Binance futures client instance
    ///
    /// # Returns
    /// A new `LPHMonitor` instance with both clients and configuration parameters configured
    pub fn new(
        config: LPHMonitorConfig,
        uniswap_client: UniswapV3PositionManager,
        binance_client: BinancePerpsClient,
    ) -> Self {
        Self {
            uniswap_client,
            binance_client,
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
    pub async fn status(&mut self) -> Result<MonitoringSnapshot> {
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
