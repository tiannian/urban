//! LPH example: run LPH monitor once and print the monitoring message (e.g. for Telegram).
//!
//! Usage: lph <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret> \
//!          <symbol> <base_token_address> <usdt_token_address>

use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use clients_binance::BinancePerpsClient;
use clients_uniswapv3::UniswapV3PositionManager;
use lph::{LPHMonitor, LPHMonitorConfig};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 9 {
        eprintln!(
            "Usage: {} <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret> <symbol> <base_token_address> <usdt_token_address>",
            args.first().map(|s| s.as_str()).unwrap_or("lph")
        );
        std::process::exit(1);
    }

    let owner = Address::from_str(args[1].trim())?;
    let contract_address = Address::from_str(args[2].trim())?;
    let rpc_url = args[3].trim();
    let api_key = args[4].trim().to_string();
    let api_secret = args[5].trim().to_string();
    let symbol = args[6].trim().to_string();
    let base_token_address = Address::from_str(args[7].trim())?;
    let usdt_token_address = Address::from_str(args[8].trim())?;

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
    let mut monitor = LPHMonitor::new(config, uniswap_client, binance_client);
    let snapshot = monitor.status().await?;
    println!("{}", snapshot.to_message(&snapshot.symbol));

    Ok(())
}
