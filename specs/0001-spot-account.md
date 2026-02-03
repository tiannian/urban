## Spot Account Specification

### Overview

This document specifies the `SpotAccount` trait used to represent a spot trading account within the system.
A spot account tracks balances for individual assets and provides a standard interface for querying balances
and performing (or simulating) exchanges between assets. All return types are wrapped in
`anyhow::Result<T>` to provide a consistent error model across implementations.

### Types and Trait Definition

The canonical Rust types and trait definition for a spot account are:

```rust
/// Asset identifier used throughout the spot account interface.
///
/// `Asset<'a>` is a newtype wrapper around `&'a str`. Implementations MUST
/// treat the inner string as a case-sensitive asset identifier (e.g. "BTC",
/// "USDT") and ensure that the same symbol always refers to the same logical
/// asset within a given context.
pub struct Asset<'a>(pub &'a str);

pub trait SpotAccount {
    /// Returns the on-ledger balance for the given `asset`.
    ///
    /// Implementations MUST:
    /// - Interpret the inner string of `asset` as a case-sensitive asset identifier
    ///   (e.g. `"BTC"`, `"USDT"`).
    /// - Return the balance (on success) as an unsigned 128-bit integer, representing the smallest
    ///   indivisible unit of the asset (e.g. "sats" for BTC, minor units for fiat).
    fn balance(&self, asset: Asset<'_>) -> anyhow::Result<u128>;

    /// Performs an exchange from one asset into another and returns the amount of `to`
    /// asset received or expected on success.
    ///
    /// All errors are represented using `anyhow::Error` via `anyhow::Result<u128>`,
    /// and concrete error semantics are defined by the implementation.
    ///
    /// Implementations MUST:
    /// - Treat the inner strings of `from` and `to` as asset identifiers consistent with `balance`.
    /// - Use `is_simulated` to distinguish between a real exchange and a dry-run:
    ///   - When `is_simulated == false`, a real exchange MUST be attempted:
    ///     - Fail with an error if the account does not have sufficient `from` balance,
    ///       or if the requested conversion is not supported.
    ///     - On `Ok(amount)`, ensure the internal state is updated atomically so that:
    ///       - The `from` balance is reduced according to the executed exchange.
    ///       - The `to` balance is increased by the returned `amount`.
    ///   - When `is_simulated == true`, the method MUST behave as a pure simulation:
    ///     - It MUST NOT mutate any balances or persistent state.
    ///     - The returned `amount` represents the expected `to` asset output under
    ///       current conditions (including fees, rounding, and slippage rules).
    /// - Ensure any rounding rules or fees are documented by the implementation.
    fn exchange(
        &self,
        from: Asset<'_>,
        to: Asset<'_>,
        is_simulated: bool,
    ) -> anyhow::Result<u128>;
}
```

### Detailed Specifications

- **Asset identification**
  - Asset identifiers are represented by the `Asset<'a>` type.
  - The inner string of `Asset<'a>` MUST use a well-defined, case-sensitive string format
    for asset identifiers.
  - The same `Asset` (or an `Asset` constructed from the same inner string) passed to `balance`
    and `exchange`
    MUST refer to the same logical asset.

- **Balances**
  - Balances returned by `balance` MUST be non-negative.
  - The returned `u128` value MUST represent the smallest indivisible unit of the asset.
  - Implementations SHOULD document any maximum supported balance per asset if it is lower than `u128::MAX`.

- **Exchange semantics**
  - For **real exchanges** (`is_simulated == false`):
    - `exchange(from, to, false)` MUST be atomic from the perspective of the account:
      - Either both debiting `from` and crediting `to` succeed, or neither is applied.
    - If the exchange requires additional context (e.g. amount, price, or external market data),
      the implementation MAY extend this trait in a separate, more specialized trait while
      preserving the core semantics defined here.
    - Errors from `exchange` SHOULD be categorized (e.g. insufficient balance, unsupported pair,
      rate unavailable) to allow callers to react appropriately.
  - For **simulated exchanges** (`is_simulated == true`):
    - The method MUST NOT mutate any balances or persistent state.
    - The returned value MUST represent the expected output amount under the same pricing,
      fee, and rounding rules used for a real exchange.
    - Implementations SHOULD guarantee that, given the same inputs and market conditions,
      the simulated result is consistent with a subsequent real exchange.
    - Implementations SHOULD document:
      - Which variables are taken into account for simulation (e.g. price, fees, slippage).
      - Any known cases where simulation results may diverge from an actual exchange
        (e.g. rapidly changing markets, liquidity constraints).

### References

- See `0000-specs-guide.md` for general specification conventions.
