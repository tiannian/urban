use reqwest::header;

use crate::utils;

/// Client for Binance perpetual futures (USDT-M) API.
pub struct BinancePerpsClient {
    client: reqwest::Client,
    api_key: String,
    api_secret: String,
    base_url: String,
}

impl BinancePerpsClient {
    pub fn new(
        client: reqwest::Client,
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

    pub async fn get_position(&self, pair: &str) -> Result<String, reqwest::Error> {
        let params: Vec<(&str, String)> = vec![
            ("symbol", pair.to_string()),
            ("timestamp", utils::binance_fapi_timestamp_ms()),
            ("recvWindow", "5000".to_string()),
        ];
        let signed_query = utils::sign_params(&self.api_secret, &params);
        let url = format!("{}/fapi/v2/positionRisk", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("X-MBX-APIKEY", &self.api_key)
            .body(signed_query)
            .send()
            .await?
            .text()
            .await?;
        Ok(resp)
    }
}
