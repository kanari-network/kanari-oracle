use crate::errors::{OracleError, Result};
use crate::fetchers::PriceFetcher;
use crate::models::PriceData;
use futures::future::join_all;
use log::{debug, error, info, warn};
use serde::Deserialize;

#[derive(Clone)]
pub struct CoinbaseFetcher {
    fetcher: PriceFetcher,
}

impl CoinbaseFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }

    /// Fetch prices from Coinbase (try Pro API first, fall back to Coinbase spot API)
    pub async fn fetch_coinbase_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        info!("Fetching Coinbase prices for symbols: {:?}", symbols);

        let coinbase_futures: Vec<_> = symbols
            .iter()
            .filter(|s| !s.is_empty())
            .map(|symbol| {
                // Clone only the inner PriceFetcher (cheap) instead of the whole wrapper.
                let fetcher = self.fetcher.clone();
                let symbol = symbol.clone();
                async move {
                    // Recreate a lightweight CoinbaseFetcher for the async task.
                    let this = CoinbaseFetcher::new(fetcher);

                    // Prefer Coinbase Pro (pro.coinbase.com API) which provides ticker/stats
                    match this.fetch_coinbase_pro_ticker(&symbol).await {
                        Ok(pd) => Ok(pd),
                        Err(e) => {
                            warn!("Coinbase Pro ticker failed for {}: {}", symbol, e);
                            // fallback to simple spot price endpoint
                            this.fetch_coinbase_spot(&symbol).await
                        }
                    }
                }
            })
            .collect();

        let mut prices = Vec::new();
        let results = join_all(coinbase_futures).await;

        for result in results {
            match result {
                Ok(price_data) => {
                    info!(
                        "Successfully fetched {} from Coinbase Pro: ${:.2}",
                        price_data.symbol, price_data.price
                    );
                    prices.push(price_data);
                }
                Err(e) => {
                    error!("All Coinbase methods failed for a symbol: {}", e);
                }
            }
        }

        if prices.is_empty() && !symbols.is_empty() {
            return Err(OracleError::ApiError(
                "Failed to fetch any prices from Coinbase".to_string(),
            ));
        }

        info!("Successfully fetched {} prices from Coinbase Pro", prices.len());
        Ok(prices)
    }

    /// Use Coinbase Pro endpoints: /products/{pair}/ticker and /products/{pair}/stats
    pub async fn fetch_coinbase_pro_ticker(&self, original_symbol: &str) -> Result<PriceData> {
        if original_symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }

        // Coinbase Pro expects pairs like BTC-USD
        let pair = if original_symbol.contains('-') {
            original_symbol.to_uppercase()
        } else {
            format!("{}-USD", original_symbol.to_uppercase())
        };

        let ticker_url = format!("https://api.pro.coinbase.com/products/{}/ticker", pair);
        let stats_url = format!("https://api.pro.coinbase.com/products/{}/stats", pair);
        let symbol = original_symbol.to_string();
        let client = self.fetcher.client().clone();

        info!(
            "Fetching Coinbase Pro ticker for: {} (ticker: {}, stats: {})",
            pair, ticker_url, stats_url
        );

        // Fetch ticker first, then stats (stats provides 24h open/volume)
        self.fetcher
            .retry_with_backoff(|| async {
                let resp = client.get(&ticker_url).send().await?;
                if !resp.status().is_success() {
                    return Err(OracleError::ApiError(format!(
                        "Coinbase Pro ticker API error for {}: {}",
                        pair,
                        resp.status()
                    )));
                }
                // Deserialize into a typed struct for safety
                #[derive(Deserialize)]
                struct CoinbaseProTicker {
                    price: String,
                }

                let ticker: CoinbaseProTicker = resp.json().await?;
                debug!("Coinbase Pro ticker parsed price: {}", ticker.price);

                let price: f64 = ticker.price.parse().map_err(|_| {
                    OracleError::ApiError(format!(
                        "Invalid price from Coinbase Pro ticker for {}: {}",
                        pair, ticker.price
                    ))
                })?;

                // Now fetch stats for 24h open (to compute change) and volume
                let resp_stats = client.get(&stats_url).send().await?;
                if !resp_stats.status().is_success() {
                    // If stats fails, still return price-only data
                    let pd = PriceData::new(symbol.to_lowercase(), price, "coinbase-pro".to_string());
                    return Ok(pd);
                }
                #[derive(Deserialize)]
                struct CoinbaseProStats {
                    open: Option<String>,
                    volume: Option<String>,
                }

                let stats: CoinbaseProStats = resp_stats.json().await?;
                debug!(
                    "Coinbase Pro stats parsed: open={:?} volume={:?}",
                    stats.open, stats.volume
                );

                let open: Option<f64> = stats.open.as_deref().and_then(|s| s.parse().ok());
                let volume: Option<f64> = stats.volume.as_deref().and_then(|s| s.parse().ok());

                let mut price_data =
                    PriceData::new(symbol.to_lowercase(), price, "coinbase-pro".to_string());

                if let Some(open_val) = open {
                    let change = price - open_val;
                    let change_pct = if open_val.abs() > f64::EPSILON {
                        (change / open_val) * 100.0
                    } else {
                        0.0
                    };
                    price_data.change_24h = Some(change);
                    price_data.change_24h_percent = Some(change_pct);
                }

                if let Some(vol) = volume {
                    price_data.volume_24h = Some(vol);
                }

                Ok(price_data)
            })
            .await
    }

    /// Fallback to Coinbase (non-pro) v2 spot price endpoint
    pub async fn fetch_coinbase_spot(&self, original_symbol: &str) -> Result<PriceData> {
        if original_symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }

        let pair = if original_symbol.contains('-') {
            original_symbol.to_uppercase()
        } else {
            format!("{}-USD", original_symbol.to_uppercase())
        };

        let url = format!("https://api.coinbase.com/v2/prices/{}/spot", pair);
        let symbol = original_symbol.to_string();
        let client = self.fetcher.client().clone();

        info!("Fetching Coinbase spot price for: {} (URL: {})", pair, url);

        self.fetcher
            .retry_with_backoff(|| async {
                let resp = client.get(&url).send().await?;
                if !resp.status().is_success() {
                    return Err(OracleError::ApiError(format!(
                        "Coinbase spot API error for {}: {}",
                        pair,
                        resp.status()
                    )));
                }
                #[derive(Deserialize)]
                struct CoinbaseSpotData {
                    #[serde(rename = "base")]
                    _base: String,
                    currency: String,
                    amount: String,
                }

                #[derive(Deserialize)]
                struct CoinbaseSpotResponse {
                    data: CoinbaseSpotData,
                }

                let v: CoinbaseSpotResponse = resp.json().await?;
                debug!(
                    "Coinbase spot parsed amount: {} {}",
                    v.data.amount, v.data.currency
                );

                let amount = v.data.amount.parse().map_err(|_| {
                    OracleError::ApiError(format!(
                        "Invalid spot price from Coinbase for {}: {}",
                        pair, v.data.amount
                    ))
                })?;

                Ok(PriceData::new(
                    symbol.to_lowercase(),
                    amount,
                    "coinbase".to_string(),
                ))
            })
            .await
    }
}
