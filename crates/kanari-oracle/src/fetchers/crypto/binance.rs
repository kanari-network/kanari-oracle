use crate::fetchers::PriceFetcher;
use crate::errors::{OracleError, Result};
use crate::models::PriceData;
use futures::future::join_all;
use log::{debug, error, info, warn};

#[derive(Clone)]
pub struct BinanceFetcher {
    fetcher: PriceFetcher,
}

impl BinanceFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
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
        let binance_futures: Vec<_> = symbols
            .iter()
            .filter(|s| !s.is_empty())
            .map(|symbol| async move {
                // Try different APIs in order of preference with proper error handling
                match self.fetch_binance_24hr_ticker(symbol).await {
                    Ok(price_data) => Ok(price_data),
                    Err(e) => {
                        warn!("Binance 24hr ticker failed for {}: {}", symbol, e);
                        self.fetch_binance_price_only(symbol).await
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

        let binance_symbol = format!("{}USDT", original_symbol.to_uppercase());
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

                let ticker_data: serde_json::Value = response.json().await?;

                debug!(
                    "Binance 24hr ticker response: {}",
                    serde_json::to_string_pretty(&ticker_data).unwrap_or_default()
                );

                let price: f64 = ticker_data["lastPrice"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| {
                        OracleError::ApiError(format!(
                            "Invalid price format from Binance 24hr for {}. Response: {}",
                            binance_symbol,
                            ticker_data
                                .get("lastPrice")
                                .map(|v| v.to_string())
                                .unwrap_or_default()
                        ))
                    })?;

                let price_change: f64 = ticker_data["priceChange"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);

                let price_change_percent: f64 = ticker_data["priceChangePercent"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);

                let volume: f64 = ticker_data["volume"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);

                info!(
                    "Parsed Binance 24hr data for {}: price={}, change={}, change%={}",
                    binance_symbol, price, price_change, price_change_percent
                );

                let mut price_data = PriceData::new(
                    symbol.to_lowercase(), // Use lowercase for consistency
                    price,
                    "binance".to_string(),
                );

                price_data.change_24h = Some(price_change);
                price_data.change_24h_percent = Some(price_change_percent);
                price_data.volume_24h = Some(volume);

                Ok(price_data)
            })
            .await
    }

    pub async fn fetch_binance_price_only(&self, symbol: &str) -> Result<PriceData> {
        if symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }

        let binance_symbol = format!("{}USDT", symbol.to_uppercase());

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

                let price_data: serde_json::Value = response.json().await?;

                let price: f64 = price_data["price"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| {
                        OracleError::ApiError(format!(
                            "Invalid price from Binance for {}: {}",
                            binance_symbol, price_data
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
