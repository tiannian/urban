//! Uniswap V3 PositionManager client and position data types.

use alloy::eips::BlockId;
use alloy::primitives::{Address, U256};
use alloy::providers::{DynProvider, Provider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contracts::{CollectParams, DecreaseLiquidityParams, IPositionManager};

/// Position data structure containing all relevant information for a position
#[derive(Debug, Clone)]
pub struct PositionData {
    /// The position NFT token ID
    pub token_id: U256,
    /// Address of token0 in the pair
    pub token0: Address,
    /// Address of token1 in the pair
    pub token1: Address,
    /// Current liquidity amount in the position
    pub liquidity: u128,
    /// Amount of token0 that would be withdrawn if all liquidity is removed
    pub withdrawable_amount0: U256,
    /// Amount of token1 that would be withdrawn if all liquidity is removed
    pub withdrawable_amount1: U256,
    /// Amount of token0 fees/rewards that can be collected
    pub collectable_amount0: U256,
    /// Amount of token1 fees/rewards that can be collected
    pub collectable_amount1: U256,
}

/// Configuration for UniswapV3PositionManager
#[derive(Debug, Clone)]
pub struct UniswapV3PositionManagerConfig {
    /// The contract address of the Uniswap V3 PositionManager contract
    pub address: Address,
    /// Provider instance for making RPC calls to the blockchain
    pub provider: Arc<DynProvider>,
}

impl Serialize for UniswapV3PositionManagerConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("UniswapV3PositionManagerConfig", 1)?;
        state.serialize_field("address", &self.address)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for UniswapV3PositionManagerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            address: Address,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(UniswapV3PositionManagerConfig {
            address: helper.address,
            provider: Arc::new(
                alloy::providers::RootProvider::<alloy::network::Ethereum>::new_http(
                    "https://eth.llamarpc.com"
                        .parse()
                        .map_err(|e| serde::de::Error::custom(format!("{}", e)))?,
                )
                .erased(),
            ),
        })
    }
}

/// UniswapV3PositionManager provides functionality to interact with Uniswap V3 PositionManager contracts
pub struct UniswapV3PositionManager {
    /// PositionManager contract instance for making RPC calls
    position_manager: IPositionManager::IPositionManagerInstance<Arc<DynProvider>>,
    /// Internal cache of position data keyed by token ID
    positions: BTreeMap<U256, PositionData>,
}

impl UniswapV3PositionManager {
    /// Creates a new `UniswapV3PositionManager` instance
    ///
    /// # Arguments
    /// * `config` - A `UniswapV3PositionManagerConfig` instance containing the contract address and provider
    ///
    /// # Returns
    /// A new `UniswapV3PositionManager` instance with the `PositionManagerInstance` initialized at the given address
    pub fn new(config: UniswapV3PositionManagerConfig) -> Self {
        let position_manager = IPositionManager::new(config.address, config.provider);
        Self {
            position_manager,
            positions: BTreeMap::new(),
        }
    }

    /// Returns a reference to the cached position data keyed by token ID.
    pub fn positions(&self) -> &BTreeMap<U256, PositionData> {
        &self.positions
    }

    /// Gets the current block number from the blockchain provider
    ///
    /// # Returns
    /// `Result<u64>` - The current block number, or an error if the request fails
    pub async fn get_block_number(&self) -> Result<u64> {
        let block = self
            .position_manager
            .provider()
            .get_block(BlockId::latest())
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to get latest block"))?;
        Ok(block.number())
    }

    /// Synchronizes the internal `BTreeMap` with the current on-chain state of all positions owned by the specified address
    ///
    /// This function performs the following steps:
    /// 1. Enumerates all positions owned by the address
    /// 2. Reads basic position information (token0, token1, liquidity)
    /// 3. Simulates liquidity withdrawal to get withdrawable amounts
    /// 4. Simulates fee collection to get collectable amounts
    /// 5. Updates the internal BTreeMap with all collected data
    ///
    /// # Arguments
    /// * `owner` - The Ethereum address that owns the Uniswap V3 positions
    ///
    /// # Returns
    /// `Result<()>` - Returns an error if any critical operation fails
    pub async fn sync_lp(&mut self, owner: Address) -> Result<()> {
        let block_number = self
            .position_manager
            .provider()
            .get_block(BlockId::latest())
            .await?
            .ok_or_else(|| anyhow::anyhow!("failed to get latest block"))?
            .number();
        let block_id = BlockId::number(block_number);

        let balance = self
            .position_manager
            .balanceOf(owner)
            .block(block_id)
            .call()
            .await?;

        for index in 0..balance.to::<u64>() {
            let token_id = self
                .position_manager
                .tokenOfOwnerByIndex(owner, U256::from(index))
                .block(block_id)
                .call()
                .await?;

            // Read position details
            let position_info = self
                .position_manager
                .positions(token_id)
                .block(block_id)
                .call()
                .await?;

            let token0 = position_info.token0;
            let token1 = position_info.token1;
            let liquidity = position_info.liquidity;

            // Step 3: Simulate liquidity withdrawal
            let mut withdrawable_amount0 = U256::ZERO;
            let mut withdrawable_amount1 = U256::ZERO;

            if liquidity > 0 {
                let decrease_params = DecreaseLiquidityParams {
                    tokenId: token_id,
                    liquidity,
                    amount0Min: U256::ZERO,
                    amount1Min: U256::ZERO,
                    deadline: U256::from(u64::MAX), // Future timestamp for simulation
                };

                match self
                    .position_manager
                    .decreaseLiquidity(decrease_params)
                    .block(block_id)
                    .call()
                    .await
                {
                    Ok(result) => {
                        withdrawable_amount0 = result.amount0;
                        withdrawable_amount1 = result.amount1;
                    }
                    Err(_) => {
                        // If simulation fails, leave amounts as zero
                    }
                }
            }

            // Step 4: Simulate fee collection
            let mut collectable_amount0 = U256::ZERO;
            let mut collectable_amount1 = U256::ZERO;

            let collect_params = CollectParams {
                tokenId: token_id,
                recipient: owner,
                amount0Max: u128::MAX,
                amount1Max: u128::MAX,
            };

            match self
                .position_manager
                .collect(collect_params)
                .block(block_id)
                .call()
                .await
            {
                Ok(result) => {
                    collectable_amount0 = result.amount0;
                    collectable_amount1 = result.amount1;
                }
                Err(_) => {
                    // If simulation fails, leave amounts as zero
                }
            }

            // Step 5: Update BTreeMap
            let position_data = PositionData {
                token_id,
                token0,
                token1,
                liquidity,
                withdrawable_amount0,
                withdrawable_amount1,
                collectable_amount0,
                collectable_amount1,
            };

            self.positions.insert(token_id, position_data);
        }

        Ok(())
    }
}
