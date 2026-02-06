use serde::{Deserialize, Serialize};

/// Configuration for BinancePerpsClient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinancePerpsClientConfig {
    /// Binance API key
    pub api_key: String,
    /// Binance API secret
    pub api_secret: String,
    /// Base URL for API endpoints
    pub base_url: String,
}
