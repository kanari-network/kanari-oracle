use std::collections::HashMap;
use chrono::{DateTime, Utc};
use log::{info, warn, error};

use crate::config::Config;
use crate::models::{PriceData, PriceFeed};
use crate::fetchers::{PriceFetcher, CryptoFetcher, StockFetcher};
use crate::errors::{OracleError, Result};

pub struct Oracle {
    config: Config,
    crypto_fetcher: CryptoFetcher,
    stock_fetcher: StockFetcher,
    price_feeds: HashMap<String, PriceFeed>,
    last_update: DateTime<Utc>,
}

impl Oracle {
    pub async fn new(config: Config) -> Result<Self> {
        config.validate()?;
        
        let price_fetcher = PriceFetcher::new(config.clone())?;
        let crypto_fetcher = CryptoFetcher::new(price_fetcher);
        
        let price_fetcher2 = PriceFetcher::new(config.clone())?;
        let stock_fetcher = StockFetcher::new(price_fetcher2);
        
        let mut oracle = Self {
            config,
            crypto_fetcher,
            stock_fetcher,
            price_feeds: HashMap::new(),
            last_update: Utc::now(),
        };
        
        // Initialize price feeds
        oracle.price_feeds.insert("crypto".to_string(), PriceFeed::new());
        oracle.price_feeds.insert("stock".to_string(), PriceFeed::new());
        
        info!("Oracle initialized successfully");
        Ok(oracle)
    }
    
    /// Update all price feeds (crypto and stocks)
    pub async fn update_all_prices(&mut self) -> Result<usize> {
        let mut total_updated = 0;
        
        // Update crypto prices
        match self.update_crypto_prices().await {
            Ok(count) => {
                total_updated += count;
                info!("Updated {} crypto prices", count);
            }
            Err(e) => {
                error!("Failed to update crypto prices: {}", e);
            }
        }
        
        // Update stock prices
        match self.update_stock_prices().await {
            Ok(count) => {
                total_updated += count;
                info!("Updated {} stock prices", count);
            }
            Err(e) => {
                error!("Failed to update stock prices: {}", e);
            }
        }
        
        self.last_update = Utc::now();
        Ok(total_updated)
    }
    
    /// Update cryptocurrency prices
    pub async fn update_crypto_prices(&mut self) -> Result<usize> {
        let prices = self.crypto_fetcher.fetch_all_crypto_prices().await?;
        let count = prices.len();
        
        let crypto_feed = self.price_feeds.get_mut("crypto")
            .ok_or_else(|| OracleError::ConfigError("Crypto feed not initialized".to_string()))?;
        
        for price_data in prices {
            crypto_feed.update_price(price_data);
        }
        
        Ok(count)
    }
    
    /// Update stock prices
    pub async fn update_stock_prices(&mut self) -> Result<usize> {
        let prices = self.stock_fetcher.fetch_all_stock_prices().await?;
        let count = prices.len();
        
        let stock_feed = self.price_feeds.get_mut("stock")
            .ok_or_else(|| OracleError::ConfigError("Stock feed not initialized".to_string()))?;
        
        for price_data in prices {
            stock_feed.update_price(price_data);
        }
        
        Ok(count)
    }
    
    /// Get cryptocurrency price by symbol
    pub async fn get_crypto_price(&self, symbol: &str) -> Result<PriceData> {
        let crypto_feed = self.price_feeds.get("crypto")
            .ok_or_else(|| OracleError::ConfigError("Crypto feed not initialized".to_string()))?;
        
        // Try to get from cache first
        if let Some(price_data) = crypto_feed.get_price(symbol) {
            return Ok(price_data.clone());
        }
        
        // If not in cache, try to fetch directly using the same logic as update_crypto_prices
        let symbol_mapping = CryptoFetcher::get_symbol_mapping();
        let coingecko_id = symbol_mapping.iter()
            .find(|(_, v)| v.as_str() == symbol.to_uppercase())
            .map(|(k, _)| k.clone());
        
        if let Some(id) = coingecko_id {
            // Try CoinGecko first
            match self.crypto_fetcher.fetch_coingecko_prices(&[id]).await {
                Ok(prices) if !prices.is_empty() => {
                    if let Some(price_data) = prices.first() {
                        return Ok(price_data.clone());
                    }
                }
                Ok(_) => {
                    info!("CoinGecko returned empty results for {}, trying Binance fallback", symbol);
                }
                Err(_) => {
                    info!("CoinGecko failed for {}, trying Binance fallback", symbol);
                }
            }
            
            // Try Binance fallback
            match self.crypto_fetcher.fetch_binance_prices(&[symbol.to_string()]).await {
                Ok(prices) if !prices.is_empty() => {
                    if let Some(price_data) = prices.first() {
                        return Ok(price_data.clone());
                    }
                }
                Ok(_) => {
                    warn!("Binance returned empty results for {}", symbol);
                }
                Err(e) => {
                    warn!("Binance fallback also failed: {}", e);
                }
            }
        }
        
        Err(OracleError::PriceNotFound(symbol.to_string()))
    }
    
    /// Get stock price by symbol
    pub async fn get_stock_price(&self, symbol: &str) -> Result<PriceData> {
        let stock_feed = self.price_feeds.get("stock")
            .ok_or_else(|| OracleError::ConfigError("Stock feed not initialized".to_string()))?;
        
        // Try to get from cache first
        if let Some(price_data) = stock_feed.get_price(symbol) {
            return Ok(price_data.clone());
        }
        
        // If not in cache, try to fetch directly
        let price_data = if self.config.stocks.alpha_vantage_api_key.is_some() {
            self.stock_fetcher.fetch_alpha_vantage_price(symbol).await?
        } else if self.config.stocks.finnhub_api_key.is_some() {
            self.stock_fetcher.fetch_finnhub_price(symbol).await?
        } else {
            self.stock_fetcher.fetch_free_stock_price(symbol).await?
        };
        
        Ok(price_data)
    }
    
    /// Get all current crypto prices
    pub fn get_all_crypto_prices(&self) -> Vec<PriceData> {
        self.price_feeds.get("crypto")
            .map(|feed| feed.get_all_prices().into_iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get all current stock prices
    pub fn get_all_stock_prices(&self) -> Vec<PriceData> {
        self.price_feeds.get("stock")
            .map(|feed| feed.get_all_prices().into_iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get available crypto symbols
    pub fn get_crypto_symbols(&self) -> Vec<String> {
        let symbol_mapping = CryptoFetcher::get_symbol_mapping();
        symbol_mapping.values().cloned().collect()
    }
    
    /// Get available stock symbols
    pub fn get_stock_symbols(&self) -> Vec<String> {
        self.config.stocks.symbols.clone()
    }
    
    /// Print current prices in a formatted table
    pub fn print_current_prices(&self) {
        println!("\n=== Current Prices (Last updated: {}) ===", self.last_update.format("%Y-%m-%d %H:%M:%S UTC"));
        
        // Print crypto prices
        let crypto_prices = self.get_all_crypto_prices();
        let crypto_is_empty = crypto_prices.is_empty();
        if !crypto_is_empty {
            println!("\n--- Cryptocurrencies ---");
            println!("{:<8} {:<12} {:<12} {:<10} {:<10}", "Symbol", "Price ($)", "24h Change", "Change %", "Source");
            println!("{}", "-".repeat(70));
            
            for price in &crypto_prices {
                let change_24h = price.change_24h.map(|c| format!("{:.2}", c)).unwrap_or_else(|| "N/A".to_string());
                let change_percent = price.change_24h_percent.map(|c| format!("{:.2}%", c)).unwrap_or_else(|| "N/A".to_string());
                
                println!("{:<8} {:<12.2} {:<12} {:<10} {:<10}", 
                    price.symbol, 
                    price.price, 
                    change_24h,
                    change_percent,
                    price.source
                );
            }
        }
        
        // Print stock prices
        let stock_prices = self.get_all_stock_prices();
        let stock_is_empty = stock_prices.is_empty();
        if !stock_is_empty {
            println!("\n--- Stocks ---");
            println!("{:<8} {:<12} {:<12} {:<10} {:<10}", "Symbol", "Price ($)", "Change", "Change %", "Source");
            println!("{}", "-".repeat(70));
            
            for price in &stock_prices {
                let change_24h = price.change_24h.map(|c| format!("{:.2}", c)).unwrap_or_else(|| "N/A".to_string());
                let change_percent = price.change_24h_percent.map(|c| format!("{:.2}%", c)).unwrap_or_else(|| "N/A".to_string());
                
                println!("{:<8} {:<12.2} {:<12} {:<10} {:<10}", 
                    price.symbol, 
                    price.price, 
                    change_24h,
                    change_percent,
                    price.source
                );
            }
        }
        
        if crypto_is_empty && stock_is_empty {
            println!("No price data available. Run update to fetch prices.");
        }
        
        println!();
    }
    
    /// Get price statistics
    pub fn get_price_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let crypto_prices = self.get_all_crypto_prices();
        let stock_prices = self.get_all_stock_prices();
        
        stats.insert("total_crypto_symbols".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(crypto_prices.len())));
        stats.insert("total_stock_symbols".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(stock_prices.len())));
        stats.insert("last_update".to_string(), 
                    serde_json::Value::String(self.last_update.to_rfc3339()));
        
        // Calculate average prices
        if !crypto_prices.is_empty() {
            let avg_crypto_price: f64 = crypto_prices.iter().map(|p| p.price).sum::<f64>() / crypto_prices.len() as f64;
            stats.insert("avg_crypto_price".to_string(), 
                        serde_json::json!(avg_crypto_price));
        }
        
        if !stock_prices.is_empty() {
            let avg_stock_price: f64 = stock_prices.iter().map(|p| p.price).sum::<f64>() / stock_prices.len() as f64;
            stats.insert("avg_stock_price".to_string(), 
                        serde_json::json!(avg_stock_price));
        }
        
        stats
    }
}