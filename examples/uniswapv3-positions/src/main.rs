//! Uniswap V3 positions example: sync and print all positions for an owner.
//! Uses Binance Futures API mark price (BNBUSDT) to compute withdrawable/collectable in USD.
//! Usage: uniswapv3-positions <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret>
//! Fetches Binance perps position for BNBUSDC using the given key/secret.
//! token0 = USD, token1 = BNB.

use alloy::network::Ethereum;
use alloy::primitives::U256;
use alloy::providers::{Provider, RootProvider};
use binance::BinancePerpsClient;
use reqwest::Client;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use uniswapv3::UniswapV3PositionManager;

const BINANCE_PREMIUM_INDEX_URL: &str =
    "https://fapi.binance.com/fapi/v1/premiumIndex?symbol=BNBUSDT";

#[derive(Debug, Deserialize)]
struct PremiumIndexResponse {
    #[serde(rename = "markPrice")]
    mark_price: String,
}

fn format_amount_18(value: U256) -> String {
    let divisor = U256::from(10u64).pow(U256::from(18u64));
    let (integer, frac) = value.div_rem(divisor);
    format!("{}.{:0>18}", integer, frac)
}

fn u256_to_f64_18(value: U256) -> f64 {
    let divisor = U256::from(10u64).pow(U256::from(18u64));
    let (integer, frac) = value.div_rem(divisor);
    let frac_str = format!("{:0>18}", frac);
    let combined = format!("{}.{}", integer, frac_str);
    combined.parse().unwrap_or(0.0)
}

async fn fetch_bnb_mark_price(client: &Client) -> Result<f64, Box<dyn std::error::Error>> {
    let resp = client
        .get(BINANCE_PREMIUM_INDEX_URL)
        .send()
        .await?
        .json::<PremiumIndexResponse>()
        .await?;
    let price: f64 = resp.mark_price.parse()?;
    Ok(price)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const BINANCE_PERPS_SYMBOL: &str = "BNBUSDC";

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!(
            "Usage: {} <owner_address> <contract_address> <rpc_url> <binance_api_key> <binance_api_secret>",
            args.first()
                .map(|s| s.as_str())
                .unwrap_or("uniswapv3-positions")
        );
        std::process::exit(1);
    }

    let owner = alloy::primitives::Address::from_str(args[1].trim())?;
    let contract_address = alloy::primitives::Address::from_str(args[2].trim())?;
    let rpc_url = args[3].trim();
    let api_key = args[4].trim().to_string();
    let api_secret = args[5].trim().to_string();

    let client = reqwest::Client::builder()
        // .resolve("fapi.binance.com", "127.0.0.1:8080".parse()?)
        .build()?;
    let client = Arc::new(client);
    let perps_config = binance::BinancePerpsClientConfig {
        client: Arc::clone(&client),
        api_key,
        api_secret,
        base_url: "https://fapi.binance.com".to_string(),
    };
    let perps_client = BinancePerpsClient::new(perps_config);
    let position_resp = perps_client.get_position(BINANCE_PERPS_SYMBOL).await?;
    println!(
        "Binance perps position ({}): {:?}",
        BINANCE_PERPS_SYMBOL, position_resp
    );

    let bnb_mark_price = fetch_bnb_mark_price(&client).await?;
    println!("BNB mark price (BNBUSDT): {}", bnb_mark_price);

    let provider = Arc::new(RootProvider::<Ethereum>::new_http(rpc_url.parse()?).erased());
    let uniswap_config = uniswapv3::UniswapV3PositionManagerConfig {
        address: contract_address,
        provider,
    };
    let mut manager = UniswapV3PositionManager::new(uniswap_config);
    manager.sync_lp(owner).await?;

    let positions = manager.positions();
    println!("Owner: {} | Positions: {}", owner, positions.len());
    for (token_id, pos) in positions {
        let w0 = u256_to_f64_18(pos.withdrawable_amount0);
        let w1 = u256_to_f64_18(pos.withdrawable_amount1);
        let c0 = u256_to_f64_18(pos.collectable_amount0);
        let c1 = u256_to_f64_18(pos.collectable_amount1);
        let withdrawable_usd = w0 + w1 * bnb_mark_price;
        let collectable_usd = c0 + c1 * bnb_mark_price;

        println!("---");
        println!("  token_id: {}", token_id);
        println!("  token0:  {} (USD)", pos.token0);
        println!("  token1:  {} (BNB)", pos.token1);
        println!("  liquidity: {}", pos.liquidity);
        println!(
            "  withdrawable_amount0 (18 decimals): {}",
            format_amount_18(pos.withdrawable_amount0)
        );
        println!(
            "  withdrawable_amount1 (18 decimals): {}",
            format_amount_18(pos.withdrawable_amount1)
        );
        println!(
            "  collectable_amount0 (18 decimals): {}",
            format_amount_18(pos.collectable_amount0)
        );
        println!(
            "  collectable_amount1 (18 decimals): {}",
            format_amount_18(pos.collectable_amount1)
        );
        println!("  withdrawable (USD): {}", withdrawable_usd);
        println!("  collectable (USD): {}", collectable_usd);
    }

    Ok(())
}
