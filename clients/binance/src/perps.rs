use std::sync::Arc;

use serde::Deserialize;

use crate::utils;

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
    pub unrealized_profit: String,
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

/// Client for Binance perpetual futures (USDT-M) API.
pub struct BinancePerpsClient {
    client: Arc<reqwest::Client>,
    api_key: String,
    api_secret: String,
    base_url: String,
}

impl BinancePerpsClient {
    pub fn new(
        client: Arc<reqwest::Client>,
        api_key: String,
        api_secret: String,
        base_url: String,
    ) -> Self {
        Self {
            client,
            api_key,
            api_secret,
            base_url,
        }
    }

    pub async fn get_position(
        &self,
        pair: &str,
    ) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
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
}
