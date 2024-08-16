use anyhow::{Context, Result};
use chrono::Utc;
use clickhouse::Client;
use std::collections::HashMap;
use url::Url;

pub struct ClickHouseClient {
    client: Client,
}

impl ClickHouseClient {
    pub fn new(url_str: &str, username: &str, password: &str) -> Result<Self> {
        let mut url = Url::parse(url_str).context("Failed to parse ClickHouse URL")?;

        // Use map_err to convert the error type
        url.set_username(username)
            .map_err(|_| anyhow::anyhow!("Failed to set username"))?;

        url.set_password(Some(password))
            .map_err(|_| anyhow::anyhow!("Failed to set password"))?;

        let client = Client::default()
            .with_url(url.as_str())
            .with_user(username)
            .with_password(password);

        Ok(Self { client })
    }

    pub async fn insert_prices(&self, prices: &HashMap<String, f64>) -> Result<()> {
        let now = Utc::now();
        let query = "INSERT INTO oracle_prices (asset_id, created_at, usd_price) VALUES";

        let values: Vec<String> = prices
            .iter()
            .map(|(asset_id, price)| format!("('{}', '{}', {})", asset_id, now, price))
            .collect();

        let full_query = format!("{} {}", query, values.join(","));
        self.client.query(&full_query).execute().await?;

        Ok(())
    }
}
