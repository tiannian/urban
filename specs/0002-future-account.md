## Future Account Specification

### Overview

This document specifies the `FutureAccount` trait used to represent a futures
trading account within the system.

A futures account tracks margin and position-related information for individual
assets (or contracts) and provides a standard interface for querying balances
and managing long/short positions. Implementations are expected to layer their
own risk, margin, and liquidation logic on top of this core interface.

### Trait Definition

The canonical Rust trait definition for a futures account is:

```rust
pub trait FutureAccount {
    /// Returns the current account balance (e.g. margin) for the given `asset`.
    ///
    /// Implementations MUST:
    /// - Interpret `asset` as a case-sensitive asset or contract identifier
    ///   (e.g. `"BTC"`, `"BTC-PERP"`).
    /// - Return the balance as an unsigned 128-bit integer, representing the
    ///   smallest indivisible unit of the asset.
    /// - Wrap the result in `anyhow::Result<u128>` to provide a consistent
    ///   error model across implementations.
    fn balance(&self, asset: &str) -> anyhow::Result<u128>;

    /// Returns or applies the 8-hour funding rate for the underlying futures
    /// market.
    ///
    /// Implementations MUST:
    /// - Return the funding rate as an unsigned 128-bit integer, typically
    ///   representing a fixed-point rate over an 8-hour window.
    /// - Wrap the result in `anyhow::Result<u128>` to provide a consistent
    ///   error model across implementations.
    ///
    /// Concrete implementations MAY choose to:
    /// - Expose the funding rate as a query-only operation, or
    /// - Apply accrued funding to the account state.
    /// The exact semantics MUST be documented by each implementation.
    fn funding_rate_8h(&self) -> anyhow::Result<u128>;

    /// Opens or increases a long (buy) position for the given `asset` by the
    /// specified `amount`.
    ///
    /// Implementations SHOULD define how notional size, leverage, and margin
    /// are specified and how this method affects internal positions and
    /// balances.
    /// All errors MUST be represented using `anyhow::Result<()>`.
    fn open_buy(&self, asset: &str, amount: u128) -> anyhow::Result<()>;

    /// Closes or reduces a long (buy) position for the given `asset` by the
    /// specified `amount`.
    ///
    /// Implementations SHOULD define how position size is determined, whether
    /// this is a full or partial close, and how realized PnL is handled.
    /// All errors MUST be represented using `anyhow::Result<()>`.
    fn close_buy(&self, asset: &str, amount: u128) -> anyhow::Result<()>;

    /// Opens or increases a short (sell) position for the given `asset` by the
    /// specified `amount`.
    ///
    /// Implementations SHOULD define how notional size, leverage, and margin
    /// are specified and how this method affects internal positions and
    /// balances.
    /// All errors MUST be represented using `anyhow::Result<()>`.
    fn open_sell(&self, asset: &str, amount: u128) -> anyhow::Result<()>;

    /// Closes or reduces a short (sell) position for the given `asset` by the
    /// specified `amount`.
    ///
    /// Implementations SHOULD define how position size is determined, whether
    /// this is a full or partial close, and how realized PnL is handled.
    /// All errors MUST be represented using `anyhow::Result<()>`.
    fn close_sell(&self, asset: &str, amount: u128) -> anyhow::Result<()>;
}
```

### Detailed Specifications

- **Asset / contract identification**
  - The `asset` parameter in `balance` MUST be treated as a case-sensitive
    identifier for the futures product (e.g. `"BTC-PERP"`, `"ETH-USD-202512"`).
  - Implementations SHOULD document the supported identifier format and any
    mapping rules between symbols and underlying markets.

- **Balances**
  - Balances returned by `balance` (inside `anyhow::Result<u128>`) MUST be
    non-negative.
  - The inner `u128` value MUST represent the smallest indivisible unit of the
    asset used for margin accounting.
  - Implementations SHOULD document:
    - Whether the balance represents available margin, total equity, or some
      other notion of account value.
    - Any maximum supported balance per asset if it is lower than `u128::MAX`.
  - Errors returned via `anyhow::Error` SHOULD be categorized (e.g. unknown
    asset, backend unavailable) so callers can react appropriately.

- **Funding rate semantics**
  - `funding_rate_8h` is intended to model the 8-hour funding mechanism
    commonly found in perpetual futures markets.
  - The inner `u128` returned (inside `anyhow::Result<u128>`) SHOULD represent
    a fixed-point funding rate over an 8-hour window; implementations MUST
    document the exact encoding (e.g. scaling factor).
  - Implementations MUST document:
    - Whether this method is read-only (query) or applies funding to the
      account.
    - How often it is expected to be called and how funding accrual is
      calculated (e.g. pro-rata over time, discrete 8h windows, etc.).

- **Position management semantics**
  - `open_buy` / `open_sell` are used to open or increase long/short positions.
    Implementations SHOULD document:
    - How position size (`amount`), price, and leverage are specified.
    - How margin is reserved and how failures (e.g. insufficient margin) are
      reported via `anyhow::Error`.
  - `close_buy` / `close_sell` are used to close or reduce existing positions.
    Implementations SHOULD document:
    - How partial vs full closes are distinguished based on `amount`.
    - How realized PnL is computed and reflected in balances.
    - How errors such as "position not found" or "insufficient position size"
      are reported via `anyhow::Error`.
  - Implementations MAY extend this trait with additional parameters and return
    types (while still using `anyhow::Result<...>` for errors) in concrete code
    while preserving the core intent of each method as described above.

### References

- See `0000-specs-guide.md` for general specification conventions.
- See `0001-spot-account.md` for the spot account counterpart to this trait.

