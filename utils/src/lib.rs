//! Shared utilities for the urban workspace.

use alloy::primitives::U256;

/// Converts a U256 value to f64, accounting for token decimals.
///
/// Values larger than `u128::MAX` are truncated; this is acceptable for f64 precision.
pub fn u256_to_f64(value: U256, decimals: u32) -> f64 {
    let value_u128 = value.to::<u128>();
    let divisor = 10_u128.pow(decimals);
    let whole_part = value_u128 / divisor;
    let fractional_part = value_u128 % divisor;
    whole_part as f64 + (fractional_part as f64 / divisor as f64)
}
