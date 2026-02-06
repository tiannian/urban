# Uniswap V3 Client Specification

## Overview

This specification describes a client for interacting with Uniswap V3 PositionManager contracts. The client provides functionality to read current LP token amounts, reward amounts, and other position-related data from Uniswap V3 positions owned by a specific address. The client maintains an internal cache of position data in a `BTreeMap` structure within the `UniswapV3PositionManager` type.

## Scope and Assumptions

- **Contract Interface**: The client uses the `PositionManagerInstance<Arc<DynProvider>>` generated from the `sol!` macro defined in `0102-contract-interface.md`. This instance provides direct access to PositionManager contract functions including `balanceOf`, `tokenOfOwnerByIndex`, `positions`, `decreaseLiquidity`, and `collect`.
- **Position Ownership**: The client operates on positions owned by a single address (the owner address).
- **Simulation Mode**: The client uses `eth_call` (simulation mode) via the provider to read position data without executing transactions on-chain.
- **Data Storage**: Position data is stored in a `BTreeMap` keyed by position token ID within the `UniswapV3PositionManager` structure.

## Terminology and Variables

- `owner`: The Ethereum address that owns the Uniswap V3 positions.
- `token_id`: The unique identifier (NFT token ID) for a Uniswap V3 position.
- `liquidity`: The amount of liquidity currently in a position (uint128).
- `token0`: Address of the first token in the position's trading pair.
- `token1`: Address of the second token in the position's trading pair.
- `amount0`: Amount of token0 that would be withdrawn or collected.
- `amount1`: Amount of token1 that would be withdrawn or collected.
- `position_data`: A data structure containing all relevant information for a position, stored in the `BTreeMap`.

## Detailed Specifications

### UniswapV3PositionManager Structure

The `UniswapV3PositionManager` type contains:
- `position_manager`: A `PositionManagerInstance<Arc<DynProvider>>` instance that provides direct access to PositionManager contract functions. This instance is generated from the `sol!` macro defined in `0102-contract-interface.md`.
- `positions`: A `BTreeMap<u256, PositionData>` that maps position token IDs to their associated data.

**UniswapV3PositionManagerConfig Structure**

The `UniswapV3PositionManagerConfig` structure contains:

- `address`: The contract address of the Uniswap V3 PositionManager contract.
- `provider`: An `Arc<DynProvider>` instance for making RPC calls to the blockchain.

The `UniswapV3PositionManagerConfig` structure must derive `serde::Serialize` and `serde::Deserialize` for serialization support.

**Constructor**

```rust
fn new(config: UniswapV3PositionManagerConfig) -> Self
```

- Creates a new `UniswapV3PositionManager` instance.
- **Parameters:**
  - `config`: A `UniswapV3PositionManagerConfig` instance containing the contract address and provider.
- **Returns:** A new `UniswapV3PositionManager` instance with the `PositionManagerInstance` initialized at the given address.

**PositionData Structure**

The `PositionData` structure contains:

- `token_id`: The position NFT token ID.
- `token0`: Address of token0 in the pair.
- `token1`: Address of token1 in the pair.
- `liquidity`: Current liquidity amount in the position.
- `withdrawable_amount0`: Amount of token0 that would be withdrawn if all liquidity is removed (from simulated `decreaseLiquidity` call).
- `withdrawable_amount1`: Amount of token1 that would be withdrawn if all liquidity is removed (from simulated `decreaseLiquidity` call).
- `collectable_amount0`: Amount of token0 fees/rewards that can be collected (from simulated `collect` call).
- `collectable_amount1`: Amount of token1 fees/rewards that can be collected (from simulated `collect` call).

### sync_lp Function

**Function Signature**

```rust
async fn sync_lp(&mut self, owner: Address) -> Result<()>
```

**Function Behavior**

The `sync_lp` function synchronizes the internal `BTreeMap` with the current on-chain state of all positions owned by the specified address. All contract function calls are made through the `position_manager` instance. The function performs the following steps:

1. **Enumerate Positions**
   - Call `self.position_manager.balanceOf(owner).call().await?` to get the total number of positions owned by the address.
   - For each index from `0` to `balanceOf(owner) - 1`:
     - Call `self.position_manager.tokenOfOwnerByIndex(owner, index).call().await?` to retrieve the position token ID.

2. **Read Position Basic Information**
   - For each token ID obtained in step 1:
     - Call `self.position_manager.positions(token_id).call().await?` to retrieve position details.
     - Extract `token0`, `token1`, and `liquidity` from the returned data.

3. **Simulate Liquidity Withdrawal**
   - For each position:
     - Prepare `DecreaseLiquidityParams` with:
       - `tokenId`: The position token ID.
       - `liquidity`: The full `liquidity` amount from step 2 (to simulate complete withdrawal).
       - `amount0Min`: 0 (minimum constraints not needed for simulation).
       - `amount1Min`: 0.
       - `deadline`: A future timestamp (not critical for simulation).
     - Call `self.position_manager.decreaseLiquidity(params).call().await?` via `eth_call` (simulation mode).
     - Extract `amount0` and `amount1` from the return values as `withdrawable_amount0` and `withdrawable_amount1`.

4. **Simulate Fee Collection**
   - For each position:
     - Prepare `CollectParams` with:
       - `tokenId`: The position token ID.
       - `recipient`: The owner address (or any address, not critical for simulation).
       - `amount0Max`: Maximum value (e.g., `u128::MAX`) to collect all available fees.
       - `amount1Max`: Maximum value (e.g., `u128::MAX`) to collect all available fees.
     - Call `self.position_manager.collect(params).call().await?` via `eth_call` (simulation mode).
     - Extract `amount0` and `amount1` from the return values as `collectable_amount0` and `collectable_amount1`.

5. **Update BTreeMap**
   - For each position processed:
     - Create or update the `PositionData` entry in the `BTreeMap` using the token ID as the key.
     - Store all collected information: `token0`, `token1`, `liquidity`, `withdrawable_amount0`, `withdrawable_amount1`, `collectable_amount0`, `collectable_amount1`.

**Error Handling**

- If `balanceOf` or `tokenOfOwnerByIndex` calls fail, the function should return an error.
- If `positions` call fails for a specific token ID, the function may skip that position and continue with others, or return an error depending on implementation policy.
- If simulation calls (`decreaseLiquidity` or `collect`) fail for a position, the function may:
  - Set the corresponding amounts to zero or a sentinel value.
  - Skip updating those fields for that position.
  - Return an error (implementation-specific).

**Concurrency Considerations**

- The function should handle multiple positions efficiently, potentially using concurrent calls where appropriate.
- The `BTreeMap` update should be atomic or properly synchronized if the function is called concurrently.

## Usage Patterns

### Initial Synchronization

To populate the position cache for the first time:

```rust
let provider: Arc<DynProvider> = ...; // Initialize provider
let position_manager_address = Address::from_str("...")?;
let config = UniswapV3PositionManagerConfig {
    address: position_manager_address,
    provider,
};
let mut manager = UniswapV3PositionManager::new(config);
manager.sync_lp(owner_address).await?;
```

### Periodic Updates

To refresh the position data:

```rust
// Called periodically (e.g., every N seconds)
manager.sync_lp(owner_address).await?;
```

### Reading Position Data

After synchronization, position data can be read from the `BTreeMap`:

```rust
for (token_id, position_data) in manager.positions.iter() {
    // Access position_data.token0, position_data.token1, etc.
}
```

## Configuration Parameters

- **Provider Configuration**
  - The `Arc<DynProvider>` passed to `new()` should be configured with the appropriate RPC endpoint URL and other connection parameters.
  - `position_manager_address`: Address of the Uniswap V3 PositionManager contract (passed to `new()`).
- **Simulation Parameters**
  - Simulation calls are made via `eth_call` through the provider. The provider configuration determines the block number (defaults to latest if not specified) and timeout settings.

## References

- `0102-contract-interface.md` for contract interface function definitions and usage patterns.
- Uniswap V3 PositionManager contract documentation for detailed behavior of `positions`, `decreaseLiquidity`, and `collect` functions.
