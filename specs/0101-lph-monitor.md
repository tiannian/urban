# LP Hedging Monitor Specification

## Overview

 This document describes how to monitor the overall account state for an LP hedging setup that combines:

- A centralized exchange (CEX) futures account (e.g., Binance Futures) holding perpetual positions and balances.
- An on-chain AMM position (e.g., Uniswap-style pool) holding `BASE` tokens and `USDT`.

 The monitoring service focuses on:

 1. Reading futures positions and balances from the CEX.
 2. Reading current `BASE` token and `USDT` balances from the AMM LP position.
 3. Computing the ratio between the on-chain `BASE` holdings and the futures position.
 4. Computing the total notional value across both accounts.

 This specification only covers **monitoring and reporting**. Hedging and rebalancing behavior is defined in dedicated hedging specifications (e.g., `0100-lp-hedging.md`).

## Scope and Assumptions

- **Futures venue**: A CEX that exposes:
  - Perpetual futures position size per symbol (e.g., `BNBUSDT_PERP`).
  - Account-level balances (e.g., USDT balance, unrealized PnL, margin balance).
- **On-chain venue**: An AMM (e.g., Uniswap V2/V3 or equivalent) where the LP position holds:
  - `BASE` tokens (e.g., `BNB`, `ETH`).
  - `USDT` as the quote/stable asset.
- **Pair type**: `BASE/USDT`, with `BASE` a volatile asset.
- **Goal**: Provide a single, consistent view of:
  - Net `BASE` exposure.
  - Total account value in `USDT`.
  - Deviation between on-chain `BASE` holdings and futures position.

## Terminology and Variables

- `base` / `BASE`: Volatile asset in the LP pair.
- `quote`: Stable asset (`USDT`).
- `futures_position`: Net position in the `BASE/USDT` perpetual futures on the CEX, in units of `BASE`.
- `futures_balance_usdt`: USDT-denominated account balance on the CEX (can be margin balance, wallet balance, or a defined metric).
- `amm_base_amount`: Quantity of `BASE` held in the AMM LP position.
- `amm_usdt_amount`: Quantity of `USDT` held in the AMM LP position.
- `base_price_usdt`: Current price of `BASE` in USDT (e.g., from oracle, CEX ticker, or AMM price).
- `base_delta_ratio`: Relative difference between `amm_base_amount` and `futures_position`.
- `total_value_usdt`: Total combined notional value in USDT across both accounts.

 Unless specified otherwise, all balances and positions are assumed to be point-in-time snapshots taken at the same monitoring tick.

## High-Level Monitoring Flow

 1. **Read CEX futures account state**
 2. **Read AMM LP token balances**
 3. **Compute base token vs. position difference ratio**
 4. **Compute total notional value across both accounts**
 5. **Emit a structured monitoring snapshot**

### 1. Read CEX Futures Account State

 The monitoring service periodically queries the configured CEX API for the target futures symbol.

- **Inputs**
  - `symbol`: Futures contract symbol (e.g., `BNBUSDT_PERP`).
  - Account/API credentials (implementation-specific; not part of this spec).

- **Required data**
  - `futures_position` (in `BASE` units, signed; positive = net long, negative = net short).
  - `futures_balance_usdt` (chosen balance metric, e.g., margin balance or wallet balance in USDT).
  - Optional: `unrealized_pnl_usdt` for more detailed reporting.

- **Output fields**
  - `futures_position`
  - `futures_balance_usdt`

### 2. Read AMM LP Token Balances

 The monitoring service queries the on-chain AMM for the configured LP position.

- **Inputs**
  - `amm_type` (e.g., `uniswap_v3`).
  - `pool_id` / `pair_address`.
  - LP identifier (e.g., NFT ID or position ID).

- **Required data**
  - Raw token amounts for the LP position:
    - `amm_base_amount`
    - `amm_usdt_amount`
  - Identification of which token is `BASE` and which is `USDT`.

- **Output fields**
  - `amm_base_amount`
  - `amm_usdt_amount`

 If the AMM uses ticks/ranges, the calculation of `amm_base_amount` and `amm_usdt_amount` must be consistent with the AMM definition at the current price.

### 3. Compute Base Token vs. Position Difference Ratio

 The monitoring service compares the on-chain `BASE` holdings to the futures position.

- **Definition**
  - `base_delta = amm_base_amount + futures_position`
    - Sign convention:
      - Positive `futures_position`: net long `BASE` on CEX.
      - Negative `futures_position`: net short `BASE` on CEX.
    - If the hedging strategy uses short futures against a long on-chain LP, `futures_position` will typically be negative.
  - `base_reference = max(|amm_base_amount|, |futures_position|, epsilon)`
    - `epsilon` is a small positive constant to avoid division by zero.
  - `base_delta_ratio = base_delta / base_reference`

- **Interpretation**
  - `base_delta_ratio ≈ 0`: on-chain `BASE` and futures position approximately offset.
  - `base_delta_ratio > 0`: net long `BASE` exposure (more `BASE` on-chain than hedged by futures).
  - `base_delta_ratio < 0`: net short `BASE` exposure (futures position magnitude larger than on-chain `BASE`).

 The exact sign conventions can be adjusted in implementation, but must be documented and consistent in the monitoring output.

### 4. Compute Total Notional Value Across Both Accounts

 To compute a unified total `USDT` value, the monitoring service needs a `BASE` price in `USDT`.

- **Inputs**
  - `base_price_usdt` from a configured source (e.g., oracle, CEX ticker, or AMM mid-price).

- **Per-account valuations**
  - **AMM side**
    - `amm_base_value_usdt = amm_base_amount * base_price_usdt`
    - `amm_total_value_usdt = amm_base_value_usdt + amm_usdt_amount`
  - **CEX side**
    - At minimum:
      - `futures_balance_usdt` (already in USDT units).
    - Optionally:
      - Include `unrealized_pnl_usdt` in a separate field or incorporate it into the effective balance, depending on risk policy.

- **Total**
  - `total_value_usdt = amm_total_value_usdt + futures_balance_usdt`

 Implementations may add more detailed breakdown fields, but the spec requires at least:

- `amm_total_value_usdt`
- `futures_balance_usdt`
- `total_value_usdt`

### 5. Emit Monitoring Snapshot

 On each monitoring tick, the service emits a structured snapshot that can be logged, stored, or exposed via an API.

- **Minimum snapshot fields**
  - `timestamp`
  - `symbol`
  - `amm_base_amount`
  - `amm_usdt_amount`
  - `futures_position`
  - `futures_balance_usdt`
  - `base_price_usdt`
  - `base_delta`
  - `base_delta_ratio`
  - `amm_total_value_usdt`
  - `total_value_usdt`

- **Usage**
  - Human operators can inspect snapshots to understand current exposure and PnL.
  - Automated systems can consume snapshots as inputs into alerting or further decision logic (e.g., triggering hedging actions when `base_delta_ratio` exceeds thresholds).

## Configuration Parameters (Monitoring)

- **General**
  - `monitor_interval`: Polling frequency for monitoring (e.g., in seconds).
- **CEX**
  - `exchange` (e.g., `binance`).
  - `symbol` (e.g., `BNBUSDT_PERP`).
  - Balance metric selection (e.g., margin balance vs. wallet balance).
- **AMM**
  - `amm_type` (e.g., `uniswap_v3`).
  - `pool_id` / `pair_address`.
  - `lp_position_id` / NFT ID.
- **Price source**
  - `price_source` (e.g., `binance_ticker`, `onchain_oracle`, `amm_mid_price`).
  - Optional smoothing or averaging parameters if using time-weighted prices.
- **Telegram notifications**
  - `telegram_enabled`: Boolean flag to enable/disable Telegram push.
  - `telegram_bot_token`: Bot token used to call the Telegram Bot API.
  - `telegram_chat_id`: Target chat or channel ID for notifications.
  - Optional: `telegram_parse_mode` (e.g., `MarkdownV2`, `HTML`) for message formatting.
  - Optional: `telegram_min_interval`: Minimum interval between messages to avoid spamming.
  - Optional: Thresholds for alerting based on `base_delta_ratio`, `total_value_usdt` drawdowns, or other risk metrics.

## Telegram Notification Specification

When `telegram_enabled` is true, the monitoring service must push selected monitoring information to Telegram using the Telegram Bot API.

### Triggers

At minimum, the following trigger types are supported:

1. **Periodic summary**
   - Sent every `telegram_min_interval` (or `monitor_interval` if no separate interval is configured).
   - Contains a snapshot of the current monitoring state.
2. **Alert on exposure deviation**
   - Triggered when `|base_delta_ratio|` exceeds a configured threshold.
   - May include both the current value and the threshold in the message.
3. **Alert on total value drawdown (optional)**
   - Triggered when `total_value_usdt` drops by more than a configured percentage or absolute amount from a reference value.

Implementations may add more trigger types but must document them alongside configuration.

### Message Content

Messages should be concise and human-readable, and may use Markdown or HTML formatting if `telegram_parse_mode` is configured.

- **Minimum fields in periodic summary message**
  - `symbol`
  - `base_price_usdt`
  - `amm_base_amount`, `amm_usdt_amount`
  - `futures_position`, `futures_balance_usdt`
  - `base_delta`, `base_delta_ratio`
  - `amm_total_value_usdt`, `total_value_usdt`

- **Alert message requirements**
  - Clearly identify the type of alert (e.g., "BASE exposure deviation", "Total value drawdown").
  - Include:
    - Current value (e.g., `base_delta_ratio`).
    - Threshold that was breached.
    - Optional: last known "normal" value or reference snapshot.

### Delivery and Reliability

- The monitoring service must:
  - Handle Telegram API errors gracefully (e.g., rate limits, network failures).
  - Log failures and, optionally, retry with backoff.
- Duplicate suppression (e.g., not sending the same alert repeatedly when conditions remain unchanged) is recommended but left to implementation.

## References

- `0100-lp-hedging.md` for hedging strategy details.
- CEX futures API documentation for position and balance queries.
- AMM documentation for LP position accounting and token amount calculations.
- Telegram Bot API documentation for sending messages and formatting options.
