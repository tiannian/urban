use std::sync::Arc;

use anyhow::Result;

use crate::config::BinancePerpsClientConfig;
use crate::types::{
    OrderResponse, OrderType, Orderbook, PlaceOrderRequest, Position, PositionSide, Side,
    TimeInForce,
};
use crate::utils;

/// Client for Binance perpetual futures (USDT-M) API.
pub struct BinancePerpsClient {
    client: Arc<reqwest::Client>,
    api_key: String,
    api_secret: String,
    base_url: String,
}

impl BinancePerpsClient {
    /// Creates a new `BinancePerpsClient` instance
    ///
    /// # Arguments
    /// * `client` - HTTP client instance for making requests
    /// * `config` - A `BinancePerpsClientConfig` instance containing all configuration parameters
    ///
    /// # Returns
    /// A new `BinancePerpsClient` instance with the provided configuration
    pub fn new(client: Arc<reqwest::Client>, config: BinancePerpsClientConfig) -> Self {
        Self {
            client,
            api_key: config.api_key,
            api_secret: config.api_secret,
            base_url: config.base_url,
        }
    }

    pub async fn get_position(&self, pair: &str) -> Result<Vec<Position>> {
        let params: Vec<(&str, String)> = vec![
            ("symbol", pair.to_string()),
            ("timestamp", utils::binance_fapi_timestamp_ms()),
        ];
        let signed_query = utils::sign_params(&self.api_secret, &params);
        let url = format!("{}/fapi/v3/positionRisk?{}", self.base_url, signed_query);
        let resp = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?
            .json::<Vec<Position>>()
            .await?;
        Ok(resp)
    }

    /// Fetches the order book (market depth) for the given symbol.
    ///
    /// Calls GET `/fapi/v1/depth`. This is a public endpoint; no API key or signature is required.
    ///
    /// # Arguments
    /// * `symbol` - Trading pair symbol (e.g. `BTCUSDT`)
    /// * `limit` - Optional number of depth levels (5, 10, 20, 50, 100, or 500)
    pub async fn get_orderbook(&self, symbol: &str, limit: Option<u16>) -> Result<Orderbook> {
        let mut query = format!("symbol={}", symbol);
        if let Some(n) = limit {
            query.push_str(&format!("&limit={}", n));
        }
        let url = format!("{}/fapi/v1/depth?{}", self.base_url, query);
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Orderbook>()
            .await?;
        Ok(resp)
    }

    /// Submits a single order to Binance POST `/fapi/v1/order`.
    pub async fn place_order(
        &self,
        symbol: &str,
        req: &PlaceOrderRequest,
    ) -> Result<OrderResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("symbol", symbol.to_string()),
            ("side", req.side.as_api_str().to_string()),
            ("positionSide", req.position_side.as_api_str().to_string()),
            ("type", req.order_type.as_api_str().to_string()),
            ("quantity", req.quantity.clone()),
            ("reduceOnly", req.reduce_only.to_string()),
            ("timeInForce", req.time_in_force.as_api_str().to_string()),
            ("timestamp", utils::binance_fapi_timestamp_ms()),
        ];
        if req.order_type == OrderType::Limit {
            if let Some(ref price) = req.price {
                params.push(("price", price.clone()));
            }
        }
        let signed_query = utils::sign_params(&self.api_secret, &params);
        let url = format!("{}/fapi/v1/order", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(signed_query)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        println!("place_order: http status={} body={}", status, body);
        let order_response: OrderResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("parse order response: {} body={}", e, body))?;
        Ok(order_response)
    }

    /// Places a limit sell at best ask (asks0) to open a short position.
    pub async fn open_sell(&self, symbol: &str, amount: &str) -> Result<OrderResponse> {
        println!(
            "open_sell: symbol={} amount={} fetching orderbook",
            symbol, amount
        );
        let orderbook = self.get_orderbook(symbol, Some(5)).await?;
        let ask = orderbook
            .asks
            .first()
            .ok_or_else(|| anyhow::anyhow!("orderbook asks empty"))?;
        let price = ask[0].clone();
        println!(
            "open_sell: symbol={} amount={} price={} placing limit sell at best ask",
            symbol, amount, price
        );
        let req = PlaceOrderRequest {
            side: Side::Sell,
            position_side: PositionSide::Short,
            order_type: OrderType::Limit,
            quantity: amount.to_string(),
            price: Some(price),
            reduce_only: false,
            time_in_force: TimeInForce::Gtc,
        };
        let resp = self.place_order(symbol, &req).await?;
        println!(
            "open_sell: symbol={} order_id={} order placed",
            symbol, resp.order_id
        );
        Ok(resp)
    }

    /// Places a limit buy at best bid (bids0), reduce-only, to close a short position.
    pub async fn close_sell(&self, symbol: &str, amount: &str) -> Result<OrderResponse> {
        println!(
            "close_sell: symbol={} amount={} fetching orderbook",
            symbol, amount
        );
        let orderbook = self.get_orderbook(symbol, Some(5)).await?;
        let bid = orderbook
            .bids
            .first()
            .ok_or_else(|| anyhow::anyhow!("orderbook bids empty"))?;
        let price = bid[0].clone();
        println!(
            "close_sell: symbol={} amount={} price={} placing limit buy at best bid (reduce-only)",
            symbol, amount, price
        );
        let req = PlaceOrderRequest {
            side: Side::Buy,
            position_side: PositionSide::Short,
            order_type: OrderType::Limit,
            quantity: amount.to_string(),
            price: Some(price),
            reduce_only: true,
            time_in_force: TimeInForce::Gtc,
        };
        let resp = self.place_order(symbol, &req).await?;
        println!(
            "close_sell: symbol={} order_id={} order placed",
            symbol, resp.order_id
        );
        Ok(resp)
    }
}
