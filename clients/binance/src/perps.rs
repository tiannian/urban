use std::sync::Arc;

use anyhow::Result;

use crate::config::BinancePerpsClientConfig;
use crate::types::{OrderResponse, OrderType, Orderbook, PlaceOrderRequest, Position};
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
            ("timestamp", utils::binance_fapi_timestamp_ms()),
        ];
        if req.order_type == OrderType::Limit {
            if let Some(ref price) = req.price {
                params.push(("price", price.clone()));
            }
        }
        let signed_query = utils::sign_params(&self.api_secret, &params);
        let url = format!("{}/fapi/v1/order", self.base_url);
        let order_response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(signed_query)
            .send()
            .await?
            .json::<OrderResponse>()
            .await?;
        Ok(order_response)
    }
}
