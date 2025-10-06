use crate::fetchers::PriceFetcher;
use crate::errors::{OracleError, Result};
use crate::models::*;
use futures::future::join_all;
use log::{error, info, warn};
use std::collections::HashSet;

pub mod coingecko;
pub mod binance;

#[derive(Clone)]
pub struct CryptoFetcher {
    fetcher: PriceFetcher,
}

impl CryptoFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }

    // Delegation methods to maintain previous API surface
    pub async fn fetch_coingecko_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        let cg = coingecko::CoinGeckoFetcher::new(self.fetcher.clone());
        cg.fetch_coingecko_prices(symbols).await
    }

    pub async fn fetch_binance_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        let b = binance::BinanceFetcher::new(self.fetcher.clone());
        b.fetch_binance_prices(symbols).await
    }

    pub async fn fetch_binance_24hr_ticker(&self, symbol: &str) -> Result<PriceData> {
        let b = binance::BinanceFetcher::new(self.fetcher.clone());
        b.fetch_binance_24hr_ticker(symbol).await
    }

    pub async fn fetch_binance_price_only(&self, symbol: &str) -> Result<PriceData> {
        let b = binance::BinanceFetcher::new(self.fetcher.clone());
        b.fetch_binance_price_only(symbol).await
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
}
