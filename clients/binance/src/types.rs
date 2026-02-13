use serde::{Deserialize, Serialize};

/// Order side. Serializes to API string `BUY` or `SELL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_api_str(self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// Position side. Serializes to API string `BOTH`, `LONG`, or `SHORT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    Both,
    Long,
    Short,
}

impl PositionSide {
    pub fn as_api_str(self) -> &'static str {
        match self {
            PositionSide::Both => "BOTH",
            PositionSide::Long => "LONG",
            PositionSide::Short => "SHORT",
        }
    }
}

/// Order type. Serializes to API string accepted by POST `/fapi/v1/order`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    Limit,
    Market,
}

impl OrderType {
    pub fn as_api_str(self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }
}

/// Time in force. Serializes to API string `GTC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    Gtc,
}

impl TimeInForce {
    pub fn as_api_str(self) -> &'static str {
        match self {
            TimeInForce::Gtc => "GTC",
        }
    }
}

/// Request parameters for placing a single order (POST `/fapi/v1/order`).
#[derive(Debug, Clone)]
pub struct PlaceOrderRequest {
    pub side: Side,
    pub position_side: PositionSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub price: Option<String>,
    pub reduce_only: bool,
    pub time_in_force: TimeInForce,
}

/// Response from Binance POST `/fapi/v1/order` (New Order).
#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "orderId")]
    pub order_id: i64,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "positionSide")]
    pub position_side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "origType")]
    pub orig_type: String,
    pub status: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cumQty")]
    pub cum_qty: String,
    #[serde(rename = "cumQuote")]
    pub cum_quote: String,
    pub price: String,
    #[serde(rename = "avgPrice")]
    pub avg_price: String,
    #[serde(rename = "stopPrice")]
    pub stop_price: String,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: bool,
    #[serde(rename = "closePosition")]
    pub close_position: bool,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "updateTime")]
    pub update_time: i64,
    #[serde(rename = "workingType")]
    pub working_type: String,
    #[serde(rename = "priceProtect")]
    pub price_protect: bool,
    #[serde(rename = "priceMatch")]
    pub price_match: String,
    #[serde(rename = "selfTradePreventionMode")]
    pub self_trade_prevention_mode: String,
    #[serde(rename = "goodTillDate")]
    pub good_till_date: Option<i64>,
}

/// Position information from Binance perpetual futures API.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub symbol: String,
    #[serde(rename = "positionSide")]
    pub position_side: String,
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

/// Order book (market depth) from Binance perpetual futures API.
#[derive(Debug, Clone, Deserialize)]
pub struct Orderbook {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: i64,
    #[serde(rename = "E")]
    pub e: i64,
    #[serde(rename = "T")]
    pub t: i64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}
