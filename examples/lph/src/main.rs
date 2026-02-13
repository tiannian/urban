//! LPH example: run LPH monitor in a loop every 1 minute and push the monitoring message via Telegram.
//!
//! Usage: lph <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret> <telegram_bot_key> <telegram_chat_id>
//!
//! Symbol and token addresses are fixed: BNBUSDC, WBNB, USDT (BSC).

use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use clients_binance::BinancePerpsClient;
use clients_telegrambot::TelegramBot;
use clients_uniswapv3::UniswapV3PositionManager;
use lph::{LPHMonitorConfig, LPHStrategy};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::Duration;

const SYMBOL: &str = "BNBUSDC";
const BASE_TOKEN_ADDRESS: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
const USDT_TOKEN_ADDRESS: &str = "0x55d398326f99059fF775485246999027B3197955";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 8 {
        eprintln!(
            "Usage: {} <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret> <telegram_bot_key> <telegram_chat_id>",
            args.first().map(|s| s.as_str()).unwrap_or("lph")
        );
        std::process::exit(1);
    }

    let owner = Address::from_str(args[1].trim())?;
    let contract_address = Address::from_str(args[2].trim())?;
    let rpc_url = args[3].trim();
    let api_key = args[4].trim().to_string();
    let api_secret = args[5].trim().to_string();
    let telegram_bot_key = args[6].trim().to_string();
    let telegram_chat_id = args[7].trim().to_string();
    let symbol = SYMBOL.to_string();
    let base_token_address = Address::from_str(BASE_TOKEN_ADDRESS)?;
    let usdt_token_address = Address::from_str(USDT_TOKEN_ADDRESS)?;

    let client = reqwest::Client::builder().build()?;
    let client = Arc::new(client);
    let perps_config = clients_binance::BinancePerpsClientConfig {
        api_key,
        api_secret,
        base_url: "https://fapi.binance.com".to_string(),
    };
    let binance_client = BinancePerpsClient::new(Arc::clone(&client), perps_config);

    let provider = Arc::new(RootProvider::<Ethereum>::new_http(rpc_url.parse()?).erased());
    let uniswap_config = clients_uniswapv3::UniswapV3PositionManagerConfig {
        address: contract_address,
    };
    let uniswap_client = UniswapV3PositionManager::new(uniswap_config, provider);

    let config = LPHMonitorConfig {
        owner,
        symbol,
        base_token_address,
        usdt_token_address,
    };
    let mut monitor = LPHStrategy::new(config, uniswap_client, binance_client);
    let telegram = TelegramBot::new(telegram_bot_key, telegram_chat_id);

    loop {
        let snapshot = monitor.status().await?;
        let message = snapshot.to_message("BNB");
        telegram.push_message(&message).await?;
        tokio::time::sleep(Duration::from_secs(90)).await;
    }
}
