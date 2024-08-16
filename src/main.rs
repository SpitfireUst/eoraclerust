use std::collections::HashMap;
use std::env;
mod clickhouse_client;

use clickhouse_client::ClickHouseClient;

mod coingecko;
mod rhttp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cg_api_key = env::var("COINGECKO_API_KEY").expect("COINGECKO_API_KEY must be set");
    let clickhouse_url = env::var("CLICKHOUSE_URL").expect("CLICKHOUSE_URL must be set");
    let clickhouse_user = env::var("CLICKHOUSE_USER").expect("CLICKHOUSE_USER must be set");
    let clickhouse_password =
        env::var("CLICKHOUSE_PASSWORD").expect("CLICKHOUSE_PASSWORD must be set");

    let http_client = rhttp::Client::new()
        .with_max_requests_per_second(0.5)
        .with_max_retries(5);

    // Get prices from CoinGecko
    let coin_gecko = coingecko::Client::new(http_client, cg_api_key);
    let cg_prices = coin_gecko
        .get_latest_prices(
            CG_ID_TO_ASSET_ID_MAP
                .keys()
                .map(|&s| s.to_string())
                .collect(),
            vec!["usd".to_string()],
        )
        .await?;

    let mut prices: HashMap<String, f64> = HashMap::new();
    for (id, inner_map) in cg_prices.iter() {
        if let Some(asset_id) = CG_ID_TO_ASSET_ID_MAP.get(id.as_str()) {
            if let Some(&price) = inner_map.get("usd") {
                prices.insert(asset_id.to_string(), price);
            }
        }
    }

    // Insert prices into ClickHouse
    let clickhouse_client =
        ClickHouseClient::new(&clickhouse_url, &clickhouse_user, &clickhouse_password)?;
    clickhouse_client.insert_prices(&prices).await?;

    for (k, v) in prices.iter() {
        println!("{} {}", k, v);
    }

    Ok(())
}

// Maps the CoinGecko API ID to Coinhall's internal asset ID (typically the lower-case ticker).
lazy_static::lazy_static! {
    static ref CG_ID_TO_ASSET_ID_MAP: HashMap<&'static str, &'static str> =
        HashMap::from([
            ("bitcoin", "btc"),
            ("ethereum", "eth"),
            ("usd-coin", "usdc"),
            ("tether", "usdt"),
            ("solana", "sol"),
            ("jupiter-exchange-solana", "jup"),
            ])
    ;
}
