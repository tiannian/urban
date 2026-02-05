# LP Hedging Specification

## Overview

 This document describes a basic hedging strategy for liquidity provider (LP) positions on AMMs such as Uniswap V3 or PancakeSwap for pairs like `ETH/USDT` or `BNB/USDT`.  
 The core idea is to dynamically short perpetual futures to hedge the price risk of the non-USDT asset component (e.g., ETH or BNB) in a concentrated liquidity position.

## Scope and Assumptions

- **Supported AMMs**: Uniswap V3-compatible or PancakeSwap-style AMMs exposing LP positions via NFTs or position identifiers.
- **Supported pairs**: Any `BASE/USDT` pair, with `BASE` being a volatile asset (e.g., `BNB`, `ETH`).  
  - Example used in this document: `BNB/USDT`.
- **Hedging venue**: Centralized exchange (e.g., Binance) offering **perpetual futures** on the same `BASE/USDT` pair.
- **Goal**: Reduce or eliminate net directional exposure to `BASE` while maintaining LP fee income.

## Terminology and Variables

- `base` / `BASE`: Volatile asset in the LP pair (e.g., `BNB` in `BNB/USDT`).
- `quote`: Stable asset in the LP pair (e.g., `USDT`).
- `lp_position`: Liquidity position in the AMM, represented by an NFT or equivalent identifier.
- `base_amount` (`bnb_amount`): Quantity of `BASE` embedded in the LP position at a given time.
- `perp_position` (`bnb_position`): Net position size of the `BASE` perpetual futures on the hedging venue, in units of `BASE`.
- `hedge_threshold`: Absolute deviation threshold (in `BASE` units) between `base_amount` and `perp_position` that triggers a hedge rebalance.

## High-Level Process

 1. **Manual LP Provisioning**
    - Operator manually provides liquidity into the target AMM pool (e.g., `BNB/USDT`) and obtains an `lp_position` identifier (e.g., Uniswap V3 position NFT).
    - The trading/hedging service is configured with:
      - Target AMM pool and `lp_position` identifier.
      - Target perpetual futures market (e.g., `BNBUSDT` perpetual on Binance).
      - `hedge_threshold` and other risk parameters.

 2. **Periodic LP State Read**
    - On a configurable schedule (e.g., every N seconds/minutes), the service:
      - Calls the AMM position API / smart contract to read:
        - `amount0`, `amount1` for the `lp_position`.
        - Token0 and token1 addresses / symbols to determine which is `BASE` and which is `QUOTE`.
      - Derives the current `base_amount` for the LP position (e.g., `bnb_amount`).
    - If the AMM uses ticks and ranges (e.g., Uniswap V3), the calculation must be consistent with the protocol’s definition of `amount0` and `amount1` at the current price.

 3. **Perpetual Futures Position Read**

    - The service queries the configured centralized exchange account (e.g., Binance Futures) to obtain:
      - Current `perp_position` in the corresponding `BASE/USDT` perpetual contract.
      - Side (long/short) and size in units of `BASE`.
    - Only positions in the specified account and symbol (e.g., `BNBUSDT_PERP`) are considered for hedging.

 4. **Deviation Check and Rebalancing Decision**

    - The service computes the deviation:
      - `deviation = base_amount - perp_position`
    - If `|deviation| <= hedge_threshold`, **no action** is taken for this cycle.
    - If `|deviation| > hedge_threshold`, a rebalance is required:
      - Target hedge: `target_perp_position = base_amount`
      - Required adjustment: `delta = target_perp_position - perp_position`
      - For an LP that is long `BASE` (positive `base_amount`), the hedge is typically a **short** `BASE/USDT` perpetual:
        - If `delta > 0`: increase short exposure (open/increase short or close part of an existing long).
        - If `delta < 0`: decrease short exposure (partially close short) or increase long exposure, depending on current sign of `perp_position`.

 5. **Execution of Hedge Adjustment**

    - The service sends an order (or set of orders) to the hedging venue to adjust the perpetual position from `perp_position` to `target_perp_position`:
      - Order type: configurable (e.g., market, limit, post-only).
      - Size: `|delta|` in units of `BASE`.
      - Side: determined by the sign of `delta` and current `perp_position`.
    - The service should handle:
      - Partial fills and re-tries according to risk configuration.
      - Basic safeguards (e.g., max notional per rebalance, slippage checks).

## Configuration Parameters (Minimum Set)

- **AMM configuration**
  - `amm_type` (e.g., `uniswap_v3`, `pancakeswap_v3`).
  - `pool_id` / `pair_address`.
  - `lp_position_id` (e.g., Uniswap V3 position NFT ID).
- **Perpetual futures configuration**
  - `exchange` (e.g., `binance`).
  - `symbol` (e.g., `BNBUSDT_PERP`).
  - Account / API key identifiers (not stored in specs; implementation-specific).
- **Hedging logic configuration**
  - `rebalance_interval` (polling frequency for LP and perp positions).
  - `hedge_threshold` (in units of `BASE`).
  - Optional: `max_hedge_notional`, `max_order_size`, allowed order types.

## Risk and Limitations (Informative)

- Basis risk between AMM spot price and perpetual futures mark/last price.
- Funding payments on perpetual contracts can impact PnL.
- Impermanent loss and fee income remain unhedged; this strategy only aims to neutralize **directional** exposure to `BASE`.
- Liquidity and execution risk on the hedging venue (slippage, partial fills).
- Smart contract and exchange counterparty risk.

## References

- Uniswap V3 position management and `amount0`/`amount1` documentation.
- Binance (or other CEX) perpetual futures API documentation for position queries and order placement.
- General literature on LP delta hedging for AMMs.
