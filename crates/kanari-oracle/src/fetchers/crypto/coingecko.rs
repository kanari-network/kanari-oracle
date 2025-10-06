use crate::fetchers::PriceFetcher;
use crate::errors::{OracleError, Result};
use crate::models::PriceData;
use log::{info};

#[derive(Clone)]
pub struct CoinGeckoFetcher {
    fetcher: PriceFetcher,
}

impl CoinGeckoFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }

    /// Fetch prices from CoinGecko API using simple price endpoint
    pub async fn fetch_coingecko_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let ids = symbols.join(",");
        let vs_currency = self.fetcher.config().crypto.default_vs_currency.clone();

        // Use simple price API which is less rate limited
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}&include_24hr_change=true",
            ids, vs_currency
        );

        info!("Fetching CoinGecko prices from: {}", url);

        // Clone API key if available
        let api_key = self.fetcher.config().crypto.coingecko_api_key.clone();
        let client = self.fetcher.client().clone();

        let response = self
            .fetcher
            .retry_with_backoff(|| async {
                let mut request = client
                    .get(&url)
                    .header(
                        "User-Agent",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                    )
                    .header("Accept", "application/json");

                // Add API key if available
                if let Some(ref key) = api_key {
                    request = request.header("x-cg-demo-api-key", key);
                }

                let response = request.send().await?;

                if !response.status().is_success() {
                    return Err(OracleError::ApiError(format!(
                        "CoinGecko API error: {}",
                        response.status()
                    )));
                }

                let price_data: serde_json::Value = response.json().await?;
                info!(
                    "CoinGecko returned data for {} coins",
                    price_data.as_object().map(|o| o.len()).unwrap_or(0)
                );
                Ok(price_data)
            })
            .await?;

        let mut prices = Vec::new();

        if let Some(obj) = response.as_object() {
            for (coin_id, data) in obj {
                if let Some(price_obj) = data.as_object() {
                    let price = price_obj
                        .get(&vs_currency)
                        .and_then(|p| p.as_f64())
                        .unwrap_or(0.0);

                    // Get percentage change (this is what CoinGecko provides)
                    let change_24h_percent = price_obj
                        .get(&format!("{}_24h_change", vs_currency))
                        .and_then(|c| c.as_f64());

                    // Calculate absolute change from percentage
                    let change_24h = change_24h_percent.map(|pct| (price * pct) / 100.0);

                    let mut price_data = PriceData::new(
                        coin_id.to_lowercase(), // Use lowercase for consistency
                        price,
                        "coingecko".to_string(),
                    );

                    price_data.change_24h = change_24h;
                    price_data.change_24h_percent = change_24h_percent;

                    prices.push(price_data);
                }
            }
        }

        info!(
            "Successfully fetched {} prices from CoinGecko",
            prices.len()
        );
        Ok(prices)
    }
}
