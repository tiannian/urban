//! Uniswap V3 positions example: sync and print all positions for an owner.
//! Usage: uniswapv3-positions <owner_address> <contract_address> <rpc_url>

use alloy::network::Ethereum;
use alloy::primitives::U256;
use alloy::providers::{Provider, RootProvider};
use std::str::FromStr;
use uniswapv3::UniswapV3PositionManager;

fn format_amount_18(value: U256) -> String {
    let divisor = U256::from(10u64).pow(U256::from(18u64));
    let (integer, frac) = value.div_rem(divisor);
    format!("{}.{:0>18}", integer, frac)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <owner_address> <contract_address> <rpc_url>",
            args.first()
                .map(|s| s.as_str())
                .unwrap_or("uniswapv3-positions")
        );
        std::process::exit(1);
    }

    let owner = alloy::primitives::Address::from_str(args[1].trim())?;
    let contract_address = alloy::primitives::Address::from_str(args[2].trim())?;
    let rpc_url = args[3].trim();

    let provider = RootProvider::<Ethereum>::new_http(rpc_url.parse()?).erased();

    let mut manager = UniswapV3PositionManager::new(contract_address, provider);
    manager.sync_lp(owner).await?;

    let positions = manager.positions();
    println!("Owner: {} | Positions: {}", owner, positions.len());
    for (token_id, pos) in positions {
        println!("---");
        println!("  token_id: {}", token_id);
        println!("  token0:  {}", pos.token0);
        println!("  token1:  {}", pos.token1);
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
    }

    Ok(())
}
