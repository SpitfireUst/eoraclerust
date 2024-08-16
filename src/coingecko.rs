use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

use crate::rhttp;

pub struct Client {
    rhttp_client: rhttp::Client,
    api_key: String,
}

impl Client {
    pub fn new(http_client: rhttp::Client, api_key: String) -> Self {
        if api_key.is_empty() {
            panic!("unable to create coingecko client, missing api key");
        }
        Client {
            rhttp_client: http_client,
            api_key,
        }
    }

    pub async fn get_latest_prices(
        &self,
        ids: Vec<String>,
        currencies: Vec<String>,
    ) -> Result<Prices, Box<dyn std::error::Error>> {
        let url = Url::parse_with_params(
            "https://pro-api.coingecko.com/api/v3/simple/price",
            &[
                ("ids", ids.join(",")),
                ("vs_currencies", currencies.join(",")),
                ("precision", "full".to_string()),
            ],
        )?;

        let req = self
            .rhttp_client
            .request(reqwest::Method::GET, url)
            .header("x-cg-pro-api-key", &self.api_key);

        let resp = self.rhttp_client.send(req).await?;
        let prices: Prices = resp.json().await?;

        Ok(prices)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prices(HashMap<String, HashMap<String, f64>>);

impl Deref for Prices {
    type Target = HashMap<String, HashMap<String, f64>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoricalPriceResponse {
    prices: Vec<[f64; 2]>,
}
