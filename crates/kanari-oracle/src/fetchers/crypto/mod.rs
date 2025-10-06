use crate::errors::Result;
use crate::fetchers::PriceFetcher;
use crate::models::*;
use futures::future::join_all;
use log::{error, info, warn};
pub mod binance;
pub mod coinbase;
pub mod coingecko;

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

    pub async fn fetch_coinbase_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        let c = coinbase::CoinbaseFetcher::new(self.fetcher.clone());
        c.fetch_coinbase_prices(symbols).await
    }

    /// Fetch comprehensive crypto data using multiple sources
    pub async fn fetch_all_crypto_prices(&self) -> Result<Vec<PriceData>> {
        let symbols = &self.fetcher.config().crypto.symbols;

        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        // CoinGecko's simple price API works without an API key, so enable it by default
        // Enable multiple free public sources by default (CoinGecko, Binance, Coinbase).
        // API keys are optional for Binance/Coinbase public endpoints, so prefer using them when available.
        let use_binance = true;
        let use_coinbase = self.fetcher.config().crypto.coinbase_api_key.is_some();

        let futures: Vec<_> = symbols
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                let s = s.to_string();
                let use_coinbase = use_coinbase;
                let use_binance = use_binance;
                async move {
                    // Build a single-symbol slice for the delegated fetchers
                    let single = vec![s.clone()];

                    // Prefer Binance as the primary source, then Coinbase, then CoinGecko
                    enum Source {
                        Binance,
                        Coinbase,
                        CoinGecko,
                    }

                    let primary_source = if use_binance {
                        Source::Binance
                    } else if use_coinbase {
                        Source::Coinbase
                    } else {
                        Source::CoinGecko
                    };

                    let primary = match primary_source {
                        Source::Binance => self.fetch_binance_prices(&single).await,
                        Source::Coinbase => self.fetch_coinbase_prices(&single).await,
                        Source::CoinGecko => self.fetch_coingecko_prices(&single).await,
                    };

                    match primary {
                        Ok(price_data) => Ok(price_data),
                        Err(mut e) => {
                            warn!("Failed to fetch price for {}: {}", s, e);

                            // Try ordered fallbacks: Binance -> Coinbase -> CoinGecko,
                            // skipping the primary we already attempted.
                            // Keep the last error to return if all fail.
                            if !matches!(primary_source, Source::Binance) && use_binance {
                                match self.fetch_binance_prices(&single).await {
                                    Ok(price_data) => {
                                        info!(
                                            "Successfully fetched {} price using Binance fallback",
                                            s
                                        );
                                        return Ok(price_data);
                                    }
                                    Err(err) => e = err,
                                }
                            }

                            if !matches!(primary_source, Source::Coinbase) && use_coinbase {
                                match self.fetch_coinbase_prices(&single).await {
                                    Ok(price_data) => {
                                        info!(
                                            "Successfully fetched {} price using Coinbase fallback",
                                            s
                                        );
                                        return Ok(price_data);
                                    }
                                    Err(err) => e = err,
                                }
                            }

                            // Always try CoinGecko last (it usually doesn't require API keys)
                            if !matches!(primary_source, Source::CoinGecko) {
                                match self.fetch_coingecko_prices(&single).await {
                                    Ok(price_data) => {
                                        info!(
                                            "Successfully fetched {} price using CoinGecko fallback",
                                            s
                                        );
                                        return Ok(price_data);
                                    }
                                    Err(err) => e = err,
                                }
                            }

                            error!("All APIs failed for {}: {}", s, e);
                            Err(e)
                        }
                    }
                }
            })
            .collect();

        let results = join_all(futures).await;
        let mut prices = Vec::new();
        for result in results {
            if let Ok(price_data) = result {
                // price_data is Vec<PriceData> (for the single symbol), so extend the final list
                prices.extend(price_data);
            }
        }

        info!("Successfully fetched {} crypto prices", prices.len());
        Ok(prices)
    }
}
