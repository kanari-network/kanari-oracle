use super::PriceFetcher;
use crate::models::*;
use crate::errors::{OracleError, Result};
use log::{info, warn, error, debug};
use futures::future::join_all;

#[derive(Clone)]
pub struct StockFetcher {
    fetcher: PriceFetcher,
}

impl StockFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }
    
    /// Fetch stock price from Alpha Vantage API
    pub async fn fetch_alpha_vantage_price(&self, symbol: &str) -> Result<PriceData> {
        if symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }
        let api_key = self.fetcher.config().stocks.alpha_vantage_api_key
            .as_ref()
            .ok_or_else(|| OracleError::ConfigError("Alpha Vantage API key not configured".to_string()))?;
        
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol, api_key
        );
        
        debug!("Fetching Alpha Vantage price for: {}", symbol);
        
        let client = self.fetcher.client().clone();
        
        self.fetcher.retry_with_backoff(|| async {
            let response = client.get(&url).send().await?;
            
            if !response.status().is_success() {
                return Err(OracleError::ApiError(
                    format!("Alpha Vantage API error: {}", response.status())
                ));
            }
            
            let stock_response: StockPriceResponse = response.json().await?;
            let quote = stock_response.global_quote;
            
            let price: f64 = quote.price.parse()
                .map_err(|_| OracleError::ApiError("Invalid price format".to_string()))?;
            
            let change: f64 = quote.change.parse().unwrap_or(0.0);
            
            let change_percent_str = quote.change_percent.trim_end_matches('%');
            let change_percent: f64 = change_percent_str.parse().unwrap_or(0.0);
            
            let mut price_data = PriceData::new(
                quote.symbol,
                price,
                "alpha_vantage".to_string(),
            );
            
            price_data.change_24h = Some(change);
            price_data.change_24h_percent = Some(change_percent);
            
            Ok(price_data)
        }).await
    }
    
    /// Fetch stock price from Finnhub API
    pub async fn fetch_finnhub_price(&self, symbol: &str) -> Result<PriceData> {
        if symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }
        let api_key = self.fetcher.config().stocks.finnhub_api_key
            .as_ref()
            .ok_or_else(|| OracleError::ConfigError("Finnhub API key not configured".to_string()))?;
        
        let url = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            symbol, api_key
        );
        
        debug!("Fetching Finnhub price for: {}", symbol);
        
        let symbol = symbol.to_string();
        let client = self.fetcher.client().clone();
        
        self.fetcher.retry_with_backoff(|| async {
            let response = client.get(&url).send().await?;
            
            if !response.status().is_success() {
                return Err(OracleError::ApiError(
                    format!("Finnhub API error: {}", response.status())
                ));
            }
            
            let quote: serde_json::Value = response.json().await?;
            
            let current_price = quote["c"].as_f64()
                .ok_or_else(|| OracleError::ApiError("Invalid price data from Finnhub".to_string()))?;
            
            let change = quote["d"].as_f64().unwrap_or(0.0);
            let change_percent = quote["dp"].as_f64().unwrap_or(0.0);
            
            let mut price_data = PriceData::new(
                symbol.to_uppercase(),
                current_price,
                "finnhub".to_string(),
            );
            
            price_data.change_24h = Some(change);
            price_data.change_24h_percent = Some(change_percent);
            
            Ok(price_data)
        }).await
    }
    
    /// Fetch price from free stock API (alternative when API keys not available)
    ///
    /// Note: Free Yahoo endpoints can be rate-limited or blocked. Prefer API-key providers
    /// (e.g., Alpha Vantage; consider adding alternatives like Twelve Data or Polygon with free tiers).
    pub async fn fetch_free_stock_price(&self, symbol: &str) -> Result<PriceData> {
        if symbol.is_empty() {
            return Err(OracleError::ApiError("Empty symbol provided".to_string()));
        }
        // Using Yahoo Finance alternative API (no API key required)
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        
        debug!("Fetching free stock price for: {}", symbol);
        
        let symbol = symbol.to_string();
        let client = self.fetcher.client().clone();
        
        self.fetcher.retry_with_backoff(|| async {
            let response = client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(OracleError::ApiError(
                    format!("Yahoo Finance API error: {}", response.status())
                ));
            }
            
            let data: serde_json::Value = response.json().await?;
            
            let result = &data["chart"]["result"][0];
            let meta = &result["meta"];
            
            let current_price = meta["regularMarketPrice"].as_f64()
                .ok_or_else(|| OracleError::ApiError("Invalid price data from Yahoo Finance".to_string()))?;
            
            let previous_close = meta["previousClose"].as_f64().unwrap_or(current_price);
            let change = current_price - previous_close;
            let change_percent = if previous_close != 0.0 {
                (change / previous_close) * 100.0
            } else {
                0.0
            };
            
            let mut price_data = PriceData::new(
                symbol.to_uppercase(),
                current_price,
                "yahoo_finance".to_string(),
            );
            
            price_data.change_24h = Some(change);
            price_data.change_24h_percent = Some(change_percent);
            
            Ok(price_data)
        }).await
    }
    
    /// Fetch all stock prices using available APIs
    pub async fn fetch_all_stock_prices(&self) -> Result<Vec<PriceData>> {
        let symbols = &self.fetcher.config().stocks.symbols;
        
        if symbols.is_empty() {
            return Ok(Vec::new());
        }
        
        let use_alpha = self.fetcher.config().stocks.alpha_vantage_api_key.is_some();
        let use_finnhub = self.fetcher.config().stocks.finnhub_api_key.is_some();

        let futures: Vec<_> = symbols
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                let s = s.to_string();
                let use_alpha = use_alpha;
                let use_finnhub = use_finnhub;
                async move {
                    let primary = if use_alpha {
                        self.fetch_alpha_vantage_price(&s).await
                    } else if use_finnhub {
                        self.fetch_finnhub_price(&s).await
                    } else {
                        self.fetch_free_stock_price(&s).await
                    };
                    match primary {
                        Ok(price_data) => Ok(price_data),
                        Err(e) => {
                            warn!("Failed to fetch price for {}: {}", s, e);
                            if use_alpha || use_finnhub {
                                match self.fetch_free_stock_price(&s).await {
                                    Ok(price_data) => {
                                        info!("Successfully fetched {} price using fallback API", s);
                                        Ok(price_data)
                                    }
                                    Err(fallback_error) => {
                                        error!("All APIs failed for {}: {} (fallback: {})", s, e, fallback_error);
                                        Err(fallback_error)
                                    }
                                }
                            } else {
                                Err(e)
                            }
                        }
                    }
                }
            })
            .collect();

        let results = join_all(futures).await;
        let mut prices = Vec::new();
        for result in results {
            if let Ok(price_data) = result {
                prices.push(price_data);
            }
        }
        
        info!("Successfully fetched {} stock prices", prices.len());
        Ok(prices)
    }
}