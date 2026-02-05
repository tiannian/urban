# Contract Interface Specification

## Overview

This specification defines the contract interfaces used for interacting with Uniswap V3 Position NFTs and the PositionManager contract. These interfaces enable querying position information, managing liquidity, and collecting fees.

## Detailed Specifications

### Position NFT Interface

The Position NFT contract implements the ERC-721 standard with additional methods for enumerating positions owned by an address.

#### Functions

**`balanceOf(address owner) returns (uint256)`**

- Returns the number of Position NFTs owned by the specified address.
- Used in conjunction with `tokenOfOwnerByIndex` to enumerate all positions owned by an address.

**`tokenOfOwnerByIndex(address owner, uint256 index) returns (uint256)`**

- Returns the token ID of the Position NFT at the given index of the owner's token list.
- Used together with `balanceOf` to list all Position NFT IDs owned by a specific address.
- Index must be less than `balanceOf(owner)`.

### PositionManager Interface

The PositionManager contract provides functions for managing liquidity positions and collecting fees.

#### Functions

**`positions(uint256 tokenId) returns (uint96 nonce, address operator, address token0, address token1, uint24 fee, int24 tickLower, int24 tickUpper, uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128, uint128 tokensOwed0, uint128 tokensOwed1)`**

- Returns detailed information about a specific position identified by `tokenId`.
- **Key return values:**
  - `token0`: Address of the first token in the pair. Used to identify which token this is.
  - `token1`: Address of the second token in the pair. Used to identify which token this is.
  - `liquidity`: The amount of liquidity currently in the position.

**`decreaseLiquidity(DecreaseLiquidityParams calldata params) returns (uint256 amount0, uint256 amount1)`**

- Decreases the liquidity of a position.
- **Parameters:**
  - `tokenId`: The ID of the position NFT.
  - `liquidity`: The amount of liquidity to remove.
  - `amount0Min`: Minimum amount of token0 to receive.
  - `amount1Min`: Minimum amount of token1 to receive.
  - `deadline`: Transaction deadline timestamp.
- **Returns:**
  - `amount0`: Amount of token0 withdrawn from the position.
  - `amount1`: Amount of token1 withdrawn from the position.
- **Usage:** When called via `eth_call`, this function can be used to simulate the transaction and read the amounts of token0 and token1 that would be withdrawn after decreasing liquidity, without actually executing the transaction.

**`collect(CollectParams calldata params) returns (uint256 amount0, uint256 amount1)`**

- Collects accumulated fees from a position.
- **Parameters:**
  - `tokenId`: The ID of the position NFT.
  - `recipient`: Address to receive the collected fees.
  - `amount0Max`: Maximum amount of token0 fees to collect.
  - `amount1Max`: Maximum amount of token1 fees to collect.
- **Returns:**
  - `amount0`: Amount of token0 fees collected.
  - `amount1`: Amount of token1 fees collected.
- **Usage:** When called via `eth_call`, this function can be used to simulate the transaction and read the amounts of token0 and token1 rewards that would be collected, without actually executing the transaction.

#### Data Structures

**`DecreaseLiquidityParams`**

```solidity
struct DecreaseLiquidityParams {
    uint256 tokenId;
    uint128 liquidity;
    uint256 amount0Min;
    uint256 amount1Min;
    uint256 deadline;
}
```

**`CollectParams`**

```solidity
struct CollectParams {
    uint256 tokenId;
    address recipient;
    uint128 amount0Max;
    uint128 amount1Max;
}
```

## Usage Patterns

### Enumerating Positions Owned by an Address

To list all Position NFT IDs owned by a specific address:

1. Call `balanceOf(address)` to get the total number of positions.
2. Iterate from index 0 to `balanceOf(address) - 1`.
3. For each index, call `tokenOfOwnerByIndex(address, index)` to get the Position NFT ID.

### Reading Position Information

To identify the tokens in a position and check its liquidity:

1. Call `positions(uint256 tokenId)` with the Position NFT ID.
2. Extract `token0` and `token1` addresses to identify which tokens are in the pair.
3. Extract `liquidity` to get the current liquidity amount.

### Simulating Liquidity Withdrawal

To preview the amounts that would be withdrawn when decreasing liquidity:

1. Prepare `DecreaseLiquidityParams` with the desired parameters.
2. Call `decreaseLiquidity(params)` via `eth_call` (simulation mode).
3. Read the returned `amount0` and `amount1` values.

### Simulating Fee Collection

To preview the reward amounts that would be collected:

1. Prepare `CollectParams` with the desired parameters.
2. Call `collect(params)` via `eth_call` (simulation mode).
3. Read the returned `amount0` and `amount1` values.
