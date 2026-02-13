# LPH Strategy Specification

## Overview

This specification describes the LPH (Liquidity / Position Hedge) strategy logic: when certain threshold conditions on `base_delta_ratio` and `base_delta` are satisfied, the strategy computes an order quantity from a value, then either opens a short (sell) or closes a short (sell) via the Binance client. The actions **open sell** and **close sell** are defined in [0104-binance-client.md](0104-binance-client.md).

## Scope and Assumptions

- **Inputs**: The strategy consumes `base_delta_ratio`, `base_delta`, and a configurable value (the “value” used for quantity). The thresholds `n` and `m` are configuration parameters.
- **Execution**: When the condition holds, the strategy uses the Binance client’s `open_sell` or `close_sell` with a symbol and a quantity string.
- **Precision**: The quantity passed to the client is the absolute value of the chosen value, expressed with precision `m` (rounded or formatted to the step or scale defined by the threshold parameter `m`).

## Terminology and Variables

- `base_delta_ratio`: A numeric ratio used as one part of the trigger condition.
- `base_delta`: A numeric delta used as the other part of the trigger condition and/or as the source value for the order quantity.
- `n`: Configurable threshold for `base_delta_ratio` (strategy parameter).
- `m`: Configurable threshold for `base_delta` (strategy parameter).
- **open sell**: Placing a limit sell order to open a short position at the best ask; see [0104-binance-client.md § open_sell](0104-binance-client.md#open_sell-function).
- **close sell**: Placing a limit sell order to close an existing short position at the best bid (reduce-only); see [0104-binance-client.md § close_sell](0104-binance-client.md#close_sell-function).

## Detailed Specifications

### Trigger Condition

The strategy executes the following logic **only when** both of the following hold:

1. `base_delta_ratio > n`
2. `base_delta > m`

If either condition is false, no order is placed.

### Quantity Computation

When the trigger condition is satisfied:

1. **Value**: Use the value in context (e.g. `base_delta` or a strategy-defined value derived from it) as the source value.
2. **Absolute value**: Take the absolute value of that value.
3. **Convert to step m**: Round or format the result to the precision defined by `m` (e.g. quantity step or scale equal to `m`). The result is the order quantity string passed to the client.

### Action Selection

Using the **original** (pre-absolute) value:

- **If the original value is greater than 0**: Call **open sell** with the configured symbol and the quantity string from the step above.
- **If the original value is less than 0**: Call **close sell** with the configured symbol and the quantity string from the step above.

If the original value is exactly zero, the spec does not require any action (no order).

### Summary Flow

1. If `base_delta_ratio <= n` or `base_delta <= m`, do nothing.
2. Otherwise:
   - Take the chosen value; compute `quantity = abs(value)` rounded to step `m`.
   - If `value > 0`: invoke **open sell** (symbol, quantity).
   - If `value < 0`: invoke **close sell** (symbol, quantity).

## References

- [0104-binance-client.md](0104-binance-client.md) — Defines `open_sell` and `close_sell` (signatures, behavior, and usage).
