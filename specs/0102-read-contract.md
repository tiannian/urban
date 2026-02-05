# LP Contract Read Specification

## Overview

This document specifies how to read and normalize LP position information from AMM contracts such as Uniswap V3 or PancakeSwap V3.  
The goal is to derive, in a consistent way, the current amounts of **base token** (e.g., `BNB`) and **USDT** held in a concentrated liquidity position, together with accrued fees and tick range, using only on-chain read-only calls and the official protocol math libraries.

This specification defines the expected behavior of a conceptual `readPositionInfo` operation; implementations may be written in Solidity, Rust, TypeScript, or any other language, but MUST follow the same semantics.

## Scope and Assumptions

- **Supported AMMs**: Uniswap V3-compatible or PancakeSwap-style concentrated liquidity AMMs that:
  - Represent LP positions as NFTs or numeric `tokenId`s.
  - Expose a `positions(uint256 tokenId)` view function on a position manager contract.
  - Expose a `slot0()` view function on each pool contract.
- **Supported pairs**:
  - Any `BASE/USDT` pair where:
    - `BASE` is a volatile asset (e.g., `BNB`, `ETH`).
    - `USDT` (or a USDT-equivalent stablecoin) is the quote asset.
- **Price and math**:
  - All price and amount calculations MUST use the **official AMM math libraries**, e.g. Uniswap V3 `TickMath` and `LiquidityAmounts`, or their equivalent on the target chain.
  - No local re-implementation of core math is allowed unless it is a direct, proven-correct port.
- **Read-only behavior**:
  - `readPositionInfo` MUST be a pure read operation:
    - It MUST NOT modify on-chain state.
    - It MUST be implementable as a sequence of `view` calls and pure math.

## Terminology and Conventions

- `token0`, `token1`:
  - The two ERC-20 tokens of the AMM pool as defined by the protocol.
  - Their order is determined by the AMM (e.g., address-sorted) and is **not** guaranteed to match `BASE` / `USDT`.
- `BASE`:
  - The volatile asset configured for the strategy (e.g., `BNB`).
- `USDT`:
  - The stable asset configured for the strategy (e.g., USDT or an equivalent).
- **Internal normalization rule** (critical):
  - In our internal definition and in all downstream logic:
    - `amount0` **always** denotes the **BASE token amount**.
    - `amount1` **always** denotes the **USDT amount**.
  - This may require swapping the raw AMM results if `token0`/`token1` are ordered differently on-chain.
- `tokensOwed0`, `tokensOwed1`:
  - The uncollected fees for `token0` and `token1` as exposed by the position manager.
  - After normalization, we refer to:
    - `fees_base`  → fees in BASE.
    - `fees_usdt` → fees in USDT.

## On-Chain Interfaces

### Position Manager: `positions(uint256 tokenId)`

Implementations MUST use the official position manager interface from the target AMM (e.g., Uniswap V3 `INonfungiblePositionManager`).  
The required function is:

```text
function positions(uint256 tokenId)
  external
  view
  returns (
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
```

- **Input**:
  - `tokenId`: the unique identifier of the LP position NFT.
- **Required outputs** for this specification:
  - `token0`, `token1`
  - `fee`
  - `tickLower`, `tickUpper`
  - `liquidity`
  - `tokensOwed0`, `tokensOwed1`
- **Pool resolution**:
  - Implementations MUST determine the corresponding pool contract for `(token0, token1, fee)` using the official factory or helper APIs:
    - Example (Uniswap V3-style): `factory.getPool(token0, token1, fee)`.
  - If the pool does not exist or returns the zero address, `readPositionInfo` MUST fail with a clear error.

### Pool: `slot0()`

Implementations MUST call `slot0()` on the resolved pool contract:

```text
function slot0()
  external
  view
  returns (
    uint160 sqrtPriceX96,
    int24 tick,
    uint16 observationIndex,
    uint16 observationCardinality,
    uint16 observationCardinalityNext,
    uint8 feeProtocol,
    bool unlocked
  );
```

- **Required outputs** for this specification:
  - `sqrtPriceX96`
  - `tick`
- The remaining fields are informative for monitoring but not strictly required for amount computation.

### Math Libraries: `TickMath` and `LiquidityAmounts`

Implementations MUST use the official math libraries from the AMM (or exact equivalents):

- `TickMath`:
  - `sqrtRatioAX96 = TickMath.getSqrtRatioAtTick(tickLower)`
  - `sqrtRatioBX96 = TickMath.getSqrtRatioAtTick(tickUpper)`
- `LiquidityAmounts`:
  - `LiquidityAmounts.getAmountsForLiquidity(sqrtPriceX96, sqrtRatioAX96, sqrtRatioBX96, liquidity)`
  - Returns `(amount0_raw, amount1_raw)` in the **AMM’s `token0` / `token1` ordering**.

Any deviation from these official formulas is out of scope for this specification.

## Canonical `readPositionInfo` Logic

This section defines the required logical steps for a `readPositionInfo(tokenId)` operation.

### Inputs

- `tokenId`:
  - The LP position identifier (NFT id) on the AMM.
- Static configuration (per strategy / deployment):
  - `base_token`: address of the BASE token (e.g., BNB).
  - `usdt_token`: address of the USDT (or USDT-equivalent) token.

### Step 1: Read Raw Position Data

1. Call `positions(tokenId)` on the official position manager.
2. Extract:
   - `token0`, `token1`
   - `fee`
   - `tickLower`, `tickUpper`
   - `liquidity`
   - `tokensOwed0`, `tokensOwed1`
3. Validate liquidity:
   - If `liquidity == 0`, the position is effectively empty:
     - Implementations MAY still return normalized ticks and fees.
     - `amount0_raw` and `amount1_raw` MUST be treated as zero for the purposes of this spec.

### Step 2: Resolve Pool and Read `slot0`

1. Resolve the pool contract for `(token0, token1, fee)` using the protocol’s official factory.
2. Call `slot0()` on the pool.
3. Extract:
   - `sqrtPriceX96`
   - `tick`
4. If `unlocked` is `false` or the pool address is invalid, `readPositionInfo` MUST fail with a clear error.

### Step 3: Compute Boundary Prices

Using `TickMath`:

1. Compute:
   - `sqrtRatioAX96 = TickMath.getSqrtRatioAtTick(tickLower)`
   - `sqrtRatioBX96 = TickMath.getSqrtRatioAtTick(tickUpper)`
2. These represent the square-root prices at the lower and upper ticks in Q96 format, consistent with the AMM’s conventions.

### Step 4: Compute Raw Token Amounts from Liquidity

Using `LiquidityAmounts`:

1. Call:
   - `LiquidityAmounts.getAmountsForLiquidity(sqrtPriceX96, sqrtRatioAX96, sqrtRatioBX96, liquidity)`
2. Receive:
   - `amount0_raw`: amount of `token0` in the position, at the current price.
   - `amount1_raw`: amount of `token1` in the position, at the current price.
3. These raw amounts are **aligned with the AMM’s `token0` / `token1` order**, not with our internal `BASE/USDT` convention.

### Step 5: Normalize to `BASE` and `USDT`

Implementations MUST normalize all amounts and fees to the internal convention:

- **Case A: `token0 == base_token` and `token1 == usdt_token`**
  - `amount_base  = amount0_raw`
  - `amount_usdt  = amount1_raw`
  - `fees_base    = tokensOwed0`
  - `fees_usdt    = tokensOwed1`
- **Case B: `token1 == base_token` and `token0 == usdt_token`**
  - `amount_base  = amount1_raw`
  - `amount_usdt  = amount0_raw`
  - `fees_base    = tokensOwed1`
  - `fees_usdt    = tokensOwed0`
- **Case C: any other ordering or unsupported pair**
  - `readPositionInfo` MUST fail with a clear error indicating that the on-chain pair does not match the configured `base_token` / `usdt_token`.

After this normalization:

- Our **internal definition** is:
  - `amount0` ≡ `amount_base` (e.g., BNB).
  - `amount1` ≡ `amount_usdt` (e.g., USDT).
- Downstream systems MUST treat:
  - `amount0` as the base token quantity.
  - `amount1` as the USDT quantity.
  - This remains true even if the AMM’s `token0`/`token1` order differs.

### Step 6: Final Output Fields

`readPositionInfo(tokenId)` MUST expose at least the following logical fields:

- **Tokens and ticks**
  - `base_token`: address of BASE.
  - `usdt_token`: address of USDT.
  - `tick_lower`: `tickLower` from `positions`.
  - `tick_upper`: `tickUpper` from `positions`.
  - `tick_current`: `tick` from `slot0`.
- **Price**
  - `sqrt_price_x96`: `sqrtPriceX96` from `slot0`.
- **Amounts (normalized)**
  - `amount0` / `amount_base`: BASE token amount in the position.
  - `amount1` / `amount_usdt`: USDT amount in the position.
- **Fees (normalized)**
  - `tokensOwed0` / `fees_base`: uncollected fees in BASE.
  - `tokensOwed1` / `fees_usdt`: uncollected fees in USDT.

Implementations MAY include additional informational fields (e.g., pool address, fee tier, observation data) as long as the core semantics above remain unchanged.

## Error Handling and Edge Cases

Implementations of `readPositionInfo` MUST handle the following cases:

- **Non-existent position**:
  - If `positions(tokenId)` reverts or indicates that the position does not exist, return a clear error.
- **Zero liquidity**:
  - `liquidity == 0` MUST result in:
    - `amount_base = 0`
    - `amount_usdt = 0`
    - Fees MAY still be non-zero and MUST be normalized as specified.
- **Unsupported pair**:
  - If neither `token0` nor `token1` matches the configured `base_token` or `usdt_token`, `readPositionInfo` MUST fail.
- **Math or range errors**:
  - If `TickMath.getSqrtRatioAtTick` or `LiquidityAmounts.getAmountsForLiquidity` revert due to invalid ticks or ranges, surface the error and do not silently continue.

## References

- Uniswap V3 Core and Periphery documentation (positions, `slot0`, `TickMath`, `LiquidityAmounts`).
- PancakeSwap V3 (or equivalent) documentation for position NFTs and read-only pool functions.
- Internal LP hedging specification (`0100-lp-hedging.md`) for downstream usage of `amount0` (BASE) and `amount1` (USDT).

