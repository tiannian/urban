use serde::Deserialize;

/// Position information from Binance perpetual futures API.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub symbol: String,
    #[serde(rename = "positionSide")]
    pub position_side: Option<String>,
    #[serde(rename = "positionAmt")]
    pub position_amt: String,
    #[serde(rename = "entryPrice")]
    pub entry_price: String,
    #[serde(rename = "breakEvenPrice")]
    pub break_even_price: String,
    #[serde(rename = "markPrice")]
    pub mark_price: String,
    #[serde(rename = "unRealizedProfit")]
    pub unrealized_pnl: String,
    #[serde(rename = "liquidationPrice")]
    pub liquidation_price: String,
    #[serde(rename = "isolatedMargin")]
    pub isolated_margin: String,
    pub notional: String,
    #[serde(rename = "marginAsset")]
    pub margin_asset: String,
    #[serde(rename = "isolatedWallet")]
    pub isolated_wallet: String,
    #[serde(rename = "initialMargin")]
    pub initial_margin: String,
    #[serde(rename = "maintMargin")]
    pub maint_margin: String,
    #[serde(rename = "positionInitialMargin")]
    pub position_initial_margin: String,
    #[serde(rename = "openOrderInitialMargin")]
    pub open_order_initial_margin: String,
    pub adl: i32,
    #[serde(rename = "bidNotional")]
    pub bid_notional: String,
    #[serde(rename = "askNotional")]
    pub ask_notional: String,
    #[serde(rename = "updateTime")]
    pub update_time: i64,
}
