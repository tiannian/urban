use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::types::Position;
use crate::utils;

/// Configuration for BinancePerpsClient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinancePerpsClientConfig {
    /// HTTP client instance for making requests
    #[serde(skip)]
    pub client: Arc<reqwest::Client>,
    /// Binance API key
    pub api_key: String,
    /// Binance API secret
    pub api_secret: String,
    /// Base URL for API endpoints
    pub base_url: String,
}

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
    /// * `config` - A `BinancePerpsClientConfig` instance containing all configuration parameters and the HTTP client instance
    ///
    /// # Returns
    /// A new `BinancePerpsClient` instance with the provided configuration
    pub fn new(config: BinancePerpsClientConfig) -> Self {
        Self {
            client: config.client,
            api_key: config.api_key,
            api_secret: config.api_secret,
            base_url: config.base_url,
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
