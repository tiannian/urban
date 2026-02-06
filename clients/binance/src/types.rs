use serde::Deserialize;

/// Position information from Binance perpetual futures API.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub position_side: Option<String>,
    pub position_amt: String,
    pub entry_price: String,
    pub break_even_price: String,
    pub mark_price: String,
    #[serde(rename = "unRealizedProfit")]
    pub unrealized_pnl: String,
    pub liquidation_price: String,
    pub isolated_margin: String,
    pub notional: String,
    pub margin_asset: String,
    pub isolated_wallet: String,
    pub initial_margin: String,
    pub maint_margin: String,
    pub position_initial_margin: String,
    pub open_order_initial_margin: String,
    pub adl: i32,
    pub bid_notional: String,
    pub ask_notional: String,
    pub update_time: i64,
}
