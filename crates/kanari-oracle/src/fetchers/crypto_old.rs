use super::PriceFetcher;
use crate::errors::{OracleError, Result};
use crate::models::*;
use futures::future::join_all;
use log::{debug, error, info, warn};
use std::collections::HashSet;

#[derive(Clone)]
pub struct CryptoFetcher {
    fetcher: PriceFetcher,
}

impl CryptoFetcher {
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

    async fn fetch_binance_24hr_ticker(&self, original_symbol: &str) -> Result<PriceData> {
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

    async fn fetch_binance_price_only(&self, symbol: &str) -> Result<PriceData> {
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

    /// Fetch comprehensive crypto data using multiple sources
    pub async fn fetch_all_crypto_prices(&self) -> Result<Vec<PriceData>> {
        let symbols = &self.fetcher.config().crypto.symbols;

        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_prices = Vec::new();

        // Try CoinGecko first for all symbols
        match self.fetch_coingecko_prices(symbols).await {
            Ok(prices) => {
                info!("Fetched {} prices from CoinGecko", prices.len());
                all_prices.extend(prices);
            }
            Err(e) => {
                warn!("CoinGecko failed: {}", e);
            }
        }

        // Try Binance for missing symbols individually with parallel execution
        let existing_symbols: HashSet<String> =
            all_prices.iter().map(|p| p.symbol.clone()).collect();

        let missing_symbols: Vec<String> = symbols
            .iter()
            .filter(|s| !s.is_empty() && !existing_symbols.contains(&s.to_lowercase()))
            .cloned()
            .collect();

        // Warn if symbols contain hyphens (likely invalid for Binance tickers)
        for symbol in &missing_symbols {
            if symbol.contains('-') {
                warn!(
                    "Symbol '{}' contains hyphens and may not work with Binance (expects ticker format like 'BTC')",
                    symbol
                );
            }
        }

        if !missing_symbols.is_empty() {
            let binance_futures: Vec<_> = missing_symbols
                .iter()
                .map(|symbol| async move {
                    match self.fetch_binance_24hr_ticker(symbol).await {
                        Ok(price_data) => Ok(price_data),
                        Err(e) => {
                            warn!("Binance 24hr ticker failed for {}: {}", symbol, e);
                            self.fetch_binance_price_only(symbol).await
                        }
                    }
                })
                .collect();

            let binance_results = join_all(binance_futures).await;
            for result in binance_results {
                match result {
                    Ok(price_data) => {
                        info!(
                            "Successfully fetched {} price from Binance: ${:.2}",
                            price_data.symbol, price_data.price
                        );
                        all_prices.push(price_data);
                    }
                    Err(e) => {
                        error!("All Binance APIs failed: {}", e);
                    }
                }
            }
        }

        if all_prices.is_empty() {
            return Err(OracleError::ApiError(
                "All crypto price sources failed".to_string(),
            ));
        }

        info!(
            "Successfully fetched {} total crypto prices",
            all_prices.len()
        );
        Ok(all_prices)
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
}
