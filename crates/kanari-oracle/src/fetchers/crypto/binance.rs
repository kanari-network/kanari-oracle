use crate::fetchers::PriceFetcher;
use crate::errors::{OracleError, Result};
use crate::models::PriceData;
use futures::future::join_all;
use log::{debug, error, info, warn};
use serde::Deserialize;

#[derive(Clone)]
pub struct BinanceFetcher {
    fetcher: PriceFetcher,
}

impl BinanceFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }

    /// Normalize a user-provided symbol into a Binance ticker symbol.
    /// E.g. "btc", "BTC/USD", "btc-usd" => "BTCUSDT"
    fn normalize_symbol_for_binance(original: &str) -> String {
        let mut s = original.to_uppercase();
        // remove common separators
        s = s.replace('-', "").replace('/', "");

        // If it's already a futures/USDT/USDC/USD pair, return as-is (prefer USDT)
        if s.ends_with("USDT") || s.ends_with("USDC") {
            return s;
        }

        // If it ends with USD (e.g. BTCUSD) treat as base and prefer USDT
        if s.ends_with("USD") {
            s = s.trim_end_matches("USD").to_string();
        }

        format!("{}USDT", s)
    }

    /// Fetch prices from Binance API with enhanced error handling
    pub async fn fetch_binance_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        // Warn if symbols contain hyphens (likely invalid for Binance tickers)
        for symbol in symbols.iter().filter(|s| !s.is_empty()) {
            if symbol.contains('-') {
                warn!(
                    "Symbol '{}' contains hyphens and may not work with Binance (expects ticker format like 'BTC')",
                    symbol
                );
            }
        }

        let mut prices = Vec::new();
        info!("Fetching Binance prices for symbols: {:?}", symbols);

        // Parallelize Binance calls for better performance
        // Clone only the inner PriceFetcher (cheap) for each task to avoid cloning the whole
        // BinanceFetcher (which may capture more state). This mirrors the pattern used
        // in the Coinbase fetcher and is slightly more efficient.
        let binance_futures: Vec<_> = symbols
            .iter()
            .filter(|s| !s.is_empty())
            .map(|symbol| {
                let fetcher = self.fetcher.clone();
                let symbol = symbol.clone();
                async move {
                    // Recreate a lightweight BinanceFetcher for the async task.
                    let this = BinanceFetcher::new(fetcher);

                    match this.fetch_binance_24hr_ticker(&symbol).await {
                        Ok(price_data) => Ok(price_data),
                        Err(e) => {
                            warn!("Binance 24hr ticker failed for {}: {}", symbol, e);
                            this.fetch_binance_price_only(&symbol).await
                        }
                    }
                }
            })
            .collect();

        let results = join_all(binance_futures).await;
        for result in results {
            match result {
                Ok(price_data) => {
                    info!(
                        "Successfully fetched {} from Binance: ${:.2}",
                        price_data.symbol, price_data.price
                    );
                    prices.push(price_data);
                }
                Err(e) => {
                    error!("All Binance methods failed: {}", e);
                }
            }
        }

        if prices.is_empty() && !symbols.is_empty() {
            return Err(OracleError::ApiError(
                "Failed to fetch any prices from Binance".to_string(),
            ));
        }

        info!("Successfully fetched {} prices from Binance", prices.len());
        Ok(prices)
    }

    pub async fn fetch_binance_24hr_ticker(&self, original_symbol: &str) -> Result<PriceData> {
        if original_symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }

        let binance_symbol = Self::normalize_symbol_for_binance(original_symbol);
        let url = format!(
            "https://api.binance.com/api/v3/ticker/24hr?symbol={}",
            binance_symbol
        );
        let symbol = original_symbol.to_string();
        let client = self.fetcher.client().clone();

        info!(
            "Fetching Binance 24hr ticker for: {} (URL: {})",
            binance_symbol, url
        );

        self.fetcher
            .retry_with_backoff(|| async {
                let response = client.get(&url).send().await?;

                if !response.status().is_success() {
                    return Err(OracleError::ApiError(format!(
                        "Binance 24hr API error for {}: {}",
                        binance_symbol,
                        response.status()
                    )));
                }

                #[derive(Deserialize)]
                struct Binance24hrTicker {
                    #[serde(rename = "lastPrice")]
                    last_price: String,
                    #[serde(rename = "priceChange")]
                    price_change: Option<String>,
                    #[serde(rename = "priceChangePercent")]
                    price_change_percent: Option<String>,
                    volume: Option<String>,
                }

                let ticker: Binance24hrTicker = response.json().await?;

                debug!(
                    "Binance 24hr ticker parsed: price={} change={:?} change%={:?} volume={:?}",
                    ticker.last_price, ticker.price_change, ticker.price_change_percent, ticker.volume
                );

                let price: f64 = ticker
                    .last_price
                    .parse()
                    .map_err(|_| {
                        OracleError::ApiError(format!(
                            "Invalid price format from Binance 24hr for {}: {}",
                            binance_symbol, ticker.last_price
                        ))
                    })?;

                // Parse optional fields to Option<f64> so we don't mask missing/invalid values as 0.0.
                let price_change: Option<f64> = ticker
                    .price_change
                    .as_deref()
                    .and_then(|s| s.parse().ok());

                let price_change_percent: Option<f64> = ticker
                    .price_change_percent
                    .as_deref()
                    .and_then(|s| s.parse().ok());

                let volume: Option<f64> = ticker
                    .volume
                    .as_deref()
                    .and_then(|s| s.parse().ok());

                info!(
                    "Parsed Binance 24hr data for {}: price={} change={:?} change%={:?}",
                    binance_symbol, price, price_change, price_change_percent
                );

                let mut price_data = PriceData::new(
                    symbol.to_lowercase(), // Use lowercase for consistency
                    price,
                    "binance".to_string(),
                );

                price_data.change_24h = price_change;
                price_data.change_24h_percent = price_change_percent;
                price_data.volume_24h = volume;

                Ok(price_data)
            })
            .await
    }

    pub async fn fetch_binance_price_only(&self, symbol: &str) -> Result<PriceData> {
        if symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }

        let binance_symbol = Self::normalize_symbol_for_binance(symbol);

        let url = format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            binance_symbol
        );

        let client = self.fetcher.client().clone();

        info!(
            "Fetching Binance price only for: {} (URL: {})",
            binance_symbol, url
        );

        self.fetcher
            .retry_with_backoff(|| async {
                let response = client.get(&url).send().await?;

                if !response.status().is_success() {
                    return Err(OracleError::ApiError(format!(
                        "Binance price API error for {}: {}",
                        binance_symbol,
                        response.status()
                    )));
                }

                #[derive(Deserialize)]
                struct BinancePriceTicker {
                    price: String,
                }

                let pt: BinancePriceTicker = response.json().await?;

                let price: f64 = pt.price.parse().map_err(|_| {
                    OracleError::ApiError(format!(
                        "Invalid price from Binance for {}: {}",
                        binance_symbol, pt.price
                    ))
                })?;

                Ok(PriceData::new(
                    symbol.to_lowercase(), // Use lowercase for consistency
                    price,
                    "binance".to_string(),
                ))
            })
            .await
    }
}
