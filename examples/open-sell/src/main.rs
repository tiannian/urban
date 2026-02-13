//! Open sell example: place a limit sell at best ask to open a short position on Binance USDT-M perps.
//!
//! Usage: open-sell <binance_api_key> <binance_api_secret> <symbol> <amount>

use std::sync::Arc;

use anyhow::Result;
use clients_binance::{BinancePerpsClient, BinancePerpsClientConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!(
            "Usage: {} <binance_api_key> <binance_api_secret> <symbol> <amount>",
            args.first().map(|s| s.as_str()).unwrap_or("open-sell")
        );
        std::process::exit(1);
    }

    let api_key = args[1].trim().to_string();
    let api_secret = args[2].trim().to_string();
    let symbol = args[3].trim();
    let amount = args[4].trim();

    let client = reqwest::Client::builder().build()?;
    let client = Arc::new(client);
    let config = BinancePerpsClientConfig {
        api_key,
        api_secret,
        base_url: "https://fapi.binance.com".to_string(),
    };
    let perps = BinancePerpsClient::new(client, config);

    let order = perps.open_sell(symbol, amount).await?;
    println!("{:?}", order);
    Ok(())
}
