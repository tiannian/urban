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
  - The monitoring service uses `BinancePerpsClient` (as defined in `0104-binance-client.md`) to read futures position data from Binance.
- **On-chain venue**: An AMM (e.g., Uniswap V2/V3 or equivalent) where the LP position holds:
  - `BASE` tokens (e.g., `BNB`, `ETH`).
  - `USDT` as the quote/stable asset.
  - The monitoring service uses `UniswapV3PositionManager` (as defined in `0103-uniswapv3-client.md`) to read LP position data from Uniswap V3.
- **Token decimals (Uniswap)**: In Uniswap, all tokens addressed by this monitor (BASE and USDT) are assumed to use **18 decimals**. Amounts read from the AMM (e.g. `withdrawable_amount0`, `withdrawable_amount1`) must be converted to decimal representation using 18 decimal places. Implementations that support other decimal conventions must document them and handle conversion consistently.
- **Pair type**: `BASE/USDT`, with `BASE` a volatile asset.
- **Goal**: Provide a single, consistent view of:
  - Net `BASE` exposure.
  - Total account value in `USDT`.
  - Deviation between on-chain `BASE` holdings and futures position.

## Terminology and Variables

- `base` / `BASE`: Volatile asset in the LP pair.
- `quote`: Stable asset (`USDT`).
- `futures_position`: Net position in the `BASE/USDT` perpetual futures on the CEX, in units of `BASE`.
- `unrealized_pnl`: Unrealized PnL of the futures position in USDT (from CEX `unrealized_pnl` field).
- `amm_base_amount`: Quantity of `BASE` held in the AMM LP position.
- `amm_usdt_amount`: Quantity of `USDT` held in the AMM LP position.
- `amm_collectable_base`: Quantity of `BASE` fees that can be collected from the AMM LP position (uncollected fees in BASE).
- `amm_collectable_usdt`: Quantity of `USDT` fees that can be collected from the AMM LP position (uncollected fees in USDT).
- `amm_collectable_value_usdt`: Total value in USDT of all collectable AMM fees; computed as `amm_collectable_base * base_price_usdt + amm_collectable_usdt` (the score/value of collectable fees).
- `base_price_usdt`: Current price of `BASE` in USDT (e.g., from oracle, CEX ticker, or AMM price).
- `base_delta_ratio`: Relative difference between `amm_base_amount` and `futures_position`.
- `total_value_usdt`: Total combined notional value in USDT across both accounts (AMM value plus unrealized PnL).

 Unless specified otherwise, all balances and positions are assumed to be point-in-time snapshots taken at the same monitoring tick.

## Detailed Specifications

### LPHStrategy Structure

The `LPHStrategy` type is the main monitoring structure that aggregates data from both the on-chain AMM position and the CEX futures account. It contains two client instances for reading data from their respective sources.

The `LPHStrategy` type contains:

- `uniswap_client`: An instance of `UniswapV3PositionManager` (as defined in `0103-uniswapv3-client.md`) used to read LP position data from Uniswap V3.
- `binance_client`: An instance of `BinancePerpsClient` (as defined in `0104-binance-client.md`) used to read futures position data from Binance.
- `owner`: The Ethereum address that owns the Uniswap V3 LP positions.
- `symbol`: The Binance futures symbol (e.g., `BTCUSDT`).
- `base_token_address`: The Ethereum address of the BASE token (e.g., BNB, ETH).
- `usdt_token_address`: The Ethereum address of the USDT token.

**LPHStrategyConfig Structure**

The `LPHStrategyConfig` structure contains:

- `owner`: The Ethereum address that owns the Uniswap V3 LP positions.
- `symbol`: The Binance futures symbol (e.g., `BTCUSDT`).
- `base_token_address`: The Ethereum address of the BASE token (e.g., BNB, ETH).
- `usdt_token_address`: The Ethereum address of the USDT token.

The `LPHStrategyConfig` structure must derive `serde::Serialize` and `serde::Deserialize` for serialization support.

**Constructor**

```rust
fn new(config: LPHStrategyConfig, uniswap_client: UniswapV3PositionManager, binance_client: BinancePerpsClient) -> Self
```

- Creates a new `LPHStrategy` instance.
- **Parameters:**
  - `config`: A `LPHStrategyConfig` instance containing all configuration parameters.
  - `uniswap_client`: An instance of `UniswapV3PositionManager` (as defined in `0103-uniswapv3-client.md`) used to read LP position data from Uniswap V3.
  - `binance_client`: An instance of `BinancePerpsClient` (as defined in `0104-binance-client.md`) used to read futures position data from Binance.
- **Returns:** A new `LPHStrategy` instance with both clients and configuration parameters configured.

### status Function

**Function Signature**

```rust
async fn status(&mut self) -> Result<MonitoringSnapshot, Box<dyn std::error::Error>>
```

**Function Behavior**

The `status` function performs a complete monitoring cycle by reading data from both clients and computing the monitoring metrics. The function uses the configuration parameters stored in the `LPHStrategy` structure (`owner`, `symbol`, `base_token_address`, `usdt_token_address`). The function performs the following steps:

1. **Read AMM LP Position Data**
   - Call `self.uniswap_client.sync_lp(self.owner).await?` to synchronize the Uniswap V3 position data.
   - Iterate through `self.uniswap_client.positions` to find the position matching `self.base_token_address` and `self.usdt_token_address`.
   - Extract `amm_base_amount` and `amm_usdt_amount` from the matching position's `withdrawable_amount0` and `withdrawable_amount1` fields.
     - Determine which token is `BASE` and which is `USDT` by comparing addresses.
     - Convert amounts to decimal representation using 18 decimals for both tokens (see Scope and Assumptions).
   - Extract `amm_collectable_base` and `amm_collectable_usdt` from the matching position's `collectable_amount0` and `collectable_amount1` fields, mapping to BASE and USDT by the same token address convention; convert to decimal representation using 18 decimals.
   - Obtain the current block number from the blockchain provider (via the Uniswap client's provider) and store it as `block_number`.

2. **Read Binance Futures Position Data**
   - Call `self.binance_client.get_position(&self.symbol).await?` to retrieve position information from Binance.
   - Parse the returned `Vec<Position>` to find the position matching `self.symbol`.
   - Extract `futures_position` from the `position_amt` field (convert from string to decimal, preserving sign).
   - Extract `unrealized_pnl` from the `unrealized_pnl` field (convert from string to decimal, in USDT).
   - Extract `base_price_usdt` from the `mark_price` field for price reference.
   - Extract `futures_timestamp` from the `update_time` field (already in milliseconds since Unix epoch).

3. **Compute Monitoring Metrics**
   - Compute `base_delta = amm_base_amount + futures_position`.
   - Compute `base_reference = max(|amm_base_amount|, |futures_position|, epsilon)` where `epsilon` is a small positive constant (e.g., `1e-8`).
   - Compute `base_delta_ratio = base_delta / base_reference`.
   - Compute `amm_base_value_usdt = amm_base_amount * base_price_usdt`.
   - Compute `amm_total_value_usdt = amm_base_value_usdt + amm_usdt_amount`.
   - Compute `amm_collectable_value_usdt = amm_collectable_base * base_price_usdt + amm_collectable_usdt` (the score/value of collectable AMM fees in USDT).
   - Compute `total_value_usdt = amm_total_value_usdt + unrealized_pnl`.

4. **Build and Return Monitoring Snapshot**
   - Create a `MonitoringSnapshot` structure containing all computed fields:
     - `block_number`: The current blockchain block number (from the on-chain data source).
     - `symbol`: The futures symbol (from `self.symbol`).
     - `amm_base_amount`: Amount of BASE tokens in the LP position.
     - `amm_usdt_amount`: Amount of USDT tokens in the LP position.
     - `amm_collectable_base`: Amount of BASE that can be collected as fees from the LP position.
     - `amm_collectable_usdt`: Amount of USDT that can be collected as fees from the LP position.
     - `amm_collectable_value_usdt`: Total value in USDT of collectable AMM fees (score).
     - `futures_position`: Net futures position in BASE units (signed).
     - `unrealized_pnl`: Unrealized PnL of the futures position in USDT.
     - `futures_timestamp`: Timestamp from the Binance position data (from `update_time` field, in milliseconds since Unix epoch).
     - `base_price_usdt`: Current BASE price in USDT.
     - `base_delta`: Net BASE exposure (`amm_base_amount + futures_position`).
     - `base_delta_ratio`: Relative deviation ratio.
     - `amm_total_value_usdt`: Total AMM position value in USDT.
     - `total_value_usdt`: Total combined value in USDT.
   - Return the snapshot.

**Returns:** A `MonitoringSnapshot` structure containing all monitoring metrics, or an error if data reading or computation fails.

**Error Handling**

- If `sync_lp` fails, the function returns an error.
- If no matching Uniswap position is found for the specified token addresses, the function returns an error.
- If `get_position` fails, the function returns an error.
- If no matching Binance position is found for the specified symbol, the function may return an error or use zero values depending on implementation policy.
- If any computation fails (e.g., division by zero despite epsilon check), the function returns an error.

**MonitoringSnapshot Structure**

The `MonitoringSnapshot` structure contains the following fields:

- `block_number`: u64 - The blockchain block number at which the on-chain LP position data was read.
- `symbol`: String - Futures symbol.
- `amm_base_amount`: Decimal or f64 - Amount of BASE tokens in LP position.
- `amm_usdt_amount`: Decimal or f64 - Amount of USDT tokens in LP position.
- `amm_collectable_base`: Decimal or f64 - Amount of BASE that can be collected as fees from the LP position.
- `amm_collectable_usdt`: Decimal or f64 - Amount of USDT that can be collected as fees from the LP position.
- `amm_collectable_value_usdt`: Decimal or f64 - Total value in USDT of collectable AMM fees (score).
- `futures_position`: Decimal or f64 - Net futures position in BASE units (positive = long, negative = short).
- `unrealized_pnl`: Decimal or f64 - Unrealized PnL of the futures position in USDT.
- `futures_timestamp`: i64 - Timestamp from Binance position data (from `update_time` field, in milliseconds since Unix epoch).
- `base_price_usdt`: Decimal or f64 - Current BASE price in USDT.
- `base_delta`: Decimal or f64 - Net BASE exposure.
- `base_delta_ratio`: Decimal or f64 - Relative deviation ratio.
- `amm_total_value_usdt`: Decimal or f64 - Total AMM position value in USDT.
- `total_value_usdt`: Decimal or f64 - Total combined value in USDT (AMM value plus unrealized PnL).

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
  - `unrealized_pnl` (unrealized PnL of the futures position in USDT).

- **Output fields**
  - `futures_position`
  - `unrealized_pnl`

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
  - Collectable (uncollected fee) amounts for the LP position:
    - `amm_collectable_base`
    - `amm_collectable_usdt`
  - Identification of which token is `BASE` and which is `USDT`.

- **Output fields**
  - `amm_base_amount`
  - `amm_usdt_amount`
  - `amm_collectable_base`
  - `amm_collectable_usdt`

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
    - `unrealized_pnl` is reported in the snapshot as a separate field (future pnl in USDT).

- **Total**
  - `total_value_usdt = amm_total_value_usdt + unrealized_pnl`

 Implementations may add more detailed breakdown fields, but the spec requires at least:

- `amm_total_value_usdt`
- `unrealized_pnl`
- `total_value_usdt`

### 5. Emit Monitoring Snapshot

 On each monitoring tick, the service emits a structured snapshot that can be logged, stored, or exposed via an API.

- **Minimum snapshot fields**
  - `block_number`
  - `symbol`
  - `amm_base_amount`
  - `amm_usdt_amount`
  - `amm_collectable_base`
  - `amm_collectable_usdt`
  - `amm_collectable_value_usdt`
  - `futures_position`
  - `unrealized_pnl`
  - `futures_timestamp`
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
  - `amm_collectable_base`, `amm_collectable_usdt`, `amm_collectable_value_usdt`
  - `futures_position`, `unrealized_pnl`
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
- `0103-uniswapv3-client.md` for UniswapV3PositionManager client interface and usage.
- `0104-binance-client.md` for BinancePerpsClient interface and usage.
- CEX futures API documentation for position and balance queries.
- AMM documentation for LP position accounting and token amount calculations.
- Telegram Bot API documentation for sending messages and formatting options.
