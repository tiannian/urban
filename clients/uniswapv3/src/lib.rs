use alloy::primitives::{Address, U256};
use alloy::providers::DynProvider;
use alloy::sol;
use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

// Generate PositionManager contract interface using alloy sol! macro
sol! {
    // PositionManager contract interface
    #[sol(rpc)]
    interface IPositionManager {
        function balanceOf(address owner) external view returns (uint256);
        function tokenOfOwnerByIndex(address owner, uint256 index) external view returns (uint256);
        function positions(uint256 tokenId) external view returns (
            uint96 nonce,
            address operator,
            address token0,
            address token1,
            uint24 fee,
            int24 tickLower,
            int24 tickUpper,
            uint128 liquidity,
            uint256 feeGrowthInside0LastX128,
            uint256 feeGrowthInside1LastX128,
            uint128 tokensOwed0,
            uint128 tokensOwed1
        );
        function decreaseLiquidity(DecreaseLiquidityParams calldata params) external returns (uint256 amount0, uint256 amount1);
        function collect(CollectParams calldata params) external returns (uint256 amount0, uint256 amount1);
    }

    struct DecreaseLiquidityParams {
        uint256 tokenId;
        uint128 liquidity;
        uint256 amount0Min;
        uint256 amount1Min;
        uint256 deadline;
    }

    struct CollectParams {
        uint256 tokenId;
        address recipient;
        uint128 amount0Max;
        uint128 amount1Max;
    }
}

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

/// UniswapV3PositionManager provides functionality to interact with Uniswap V3 PositionManager contracts
pub struct UniswapV3PositionManager {
    /// PositionManager contract instance for making RPC calls
    position_manager: IPositionManager::IPositionManagerInstance<Arc<DynProvider>>,
    /// Internal cache of position data keyed by token ID
    pub positions: BTreeMap<U256, PositionData>,
}

impl UniswapV3PositionManager {
    /// Creates a new `UniswapV3PositionManager` instance
    ///
    /// # Arguments
    /// * `address` - The contract address of the Uniswap V3 PositionManager contract
    /// * `provider` - An `Arc<DynProvider>` instance for making RPC calls to the blockchain
    ///
    /// # Returns
    /// A new `UniswapV3PositionManager` instance with the `PositionManagerInstance` initialized at the given address
    pub fn new(address: Address, provider: Arc<DynProvider>) -> Self {
        let position_manager = IPositionManager::new(address, provider);
        Self {
            position_manager,
            positions: BTreeMap::new(),
        }
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
        // Step 1: Enumerate positions
        let balance = self.position_manager.balanceOf(owner).call().await?;

        let mut token_ids = Vec::new();
        for index in 0..balance.to::<u64>() {
            let token_id = self
                .position_manager
                .tokenOfOwnerByIndex(owner, U256::from(index))
                .call()
                .await?;
            token_ids.push(token_id);
        }

        // Step 2: Read position basic information and simulate operations
        for token_id in token_ids {
            // Read position details
            let position_info = self.position_manager.positions(token_id).call().await?;

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

            match self.position_manager.collect(collect_params).call().await {
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
