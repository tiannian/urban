use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FundingRate {
    symbol: String,
    #[serde(rename = "fundingTime")]
    funding_time: u64,
    #[serde(rename = "fundingRate")]
    funding_rate: String,
    #[serde(rename = "markPrice")]
    mark_price: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <symbol>", args[0]);
        std::process::exit(1);
    }

    let symbol = &args[1];
    let url = format!(
        "https://fapi.binance.com/fapi/v1/fundingRate?symbol={}&limit=10",
        symbol
    );

    let response = reqwest::get(&url).await?;
    let funding_rates: Vec<FundingRate> = response.json().await?;

    for rate in funding_rates {
        println!("Symbol: {}", rate.symbol);
        println!("Funding Time: {}", rate.funding_time);
        println!("Funding Rate: {}", rate.funding_rate);
        println!("Mark Price: {}", rate.mark_price);
        println!("---");
    }

    Ok(())
}
