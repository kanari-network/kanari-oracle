use super::PriceFetcher;
use crate::models::*;
use crate::errors::{OracleError, Result};
use log::{info, warn, error, debug};

pub struct StockFetcher {
    fetcher: PriceFetcher,
}

impl StockFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }
    
    /// Fetch stock price from Alpha Vantage API
    pub async fn fetch_alpha_vantage_price(&self, symbol: &str) -> Result<PriceData> {
        let api_key = self.fetcher.config().stocks.alpha_vantage_api_key
            .as_ref()
            .ok_or_else(|| OracleError::ConfigError("Alpha Vantage API key not configured".to_string()))?;
        
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol, api_key
        );
        
        debug!("Fetching Alpha Vantage price for: {}", symbol);
        
        let url = url.clone();
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
        let api_key = self.fetcher.config().stocks.finnhub_api_key
            .as_ref()
            .ok_or_else(|| OracleError::ConfigError("Finnhub API key not configured".to_string()))?;
        
        let url = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            symbol, api_key
        );
        
        debug!("Fetching Finnhub price for: {}", symbol);
        
        let url = url.clone();
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
    pub async fn fetch_free_stock_price(&self, symbol: &str) -> Result<PriceData> {
        // Using Yahoo Finance alternative API (no API key required)
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        
        debug!("Fetching free stock price for: {}", symbol);
        
        let url = url.clone();
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
        
        let mut prices = Vec::new();
        
        for symbol in symbols {
            // Try different APIs in order of preference
            let price_result = if self.fetcher.config().stocks.alpha_vantage_api_key.is_some() {
                self.fetch_alpha_vantage_price(symbol).await
            } else if self.fetcher.config().stocks.finnhub_api_key.is_some() {
                self.fetch_finnhub_price(symbol).await
            } else {
                self.fetch_free_stock_price(symbol).await
            };
            
            match price_result {
                Ok(price_data) => {
                    prices.push(price_data);
                }
                Err(e) => {
                    warn!("Failed to fetch price for {}: {}", symbol, e);
                    
                    // Try fallback to free API if paid APIs fail
                    if self.fetcher.config().stocks.alpha_vantage_api_key.is_some() || 
                       self.fetcher.config().stocks.finnhub_api_key.is_some() {
                        match self.fetch_free_stock_price(symbol).await {
                            Ok(price_data) => {
                                info!("Successfully fetched {} price using fallback API", symbol);
                                prices.push(price_data);
                            }
                            Err(fallback_error) => {
                                error!("All APIs failed for {}: {} (fallback: {})", symbol, e, fallback_error);
                            }
                        }
                    }
                }
            }
        }
        
        info!("Successfully fetched {} stock prices", prices.len());
        Ok(prices)
    }
}