use super::PriceFetcher;
use crate::models::*;
use crate::errors::{OracleError, Result};
use log::{info, warn, error, debug};
use std::collections::HashMap;

pub struct CryptoFetcher {
    fetcher: PriceFetcher,
}

impl CryptoFetcher {
    pub fn new(fetcher: PriceFetcher) -> Self {
        Self { fetcher }
    }
    
    /// Fetch prices from CoinGecko API using simple price endpoint
    pub async fn fetch_coingecko_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
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
        
        let response = self.fetcher.retry_with_backoff(|| async {
            let mut request = client.get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Accept", "application/json");
            
            // Add API key if available
            if let Some(ref key) = api_key {
                request = request.header("x-cg-demo-api-key", key);
            }
            
            let response = request.send().await?;
            
            if !response.status().is_success() {
                return Err(OracleError::ApiError(
                    format!("CoinGecko API error: {}", response.status())
                ));
            }
            
            let price_data: serde_json::Value = response.json().await?;
            info!("CoinGecko returned data for {} coins", price_data.as_object().map(|o| o.len()).unwrap_or(0));
            Ok(price_data)
        }).await?;
        
        let mut prices = Vec::new();
        let symbol_mapping = Self::get_symbol_mapping();
        
        if let Some(obj) = response.as_object() {
            for (coin_id, data) in obj {
                if let Some(symbol) = symbol_mapping.get(coin_id) {
                    if let Some(price_obj) = data.as_object() {
                        let price = price_obj.get(&vs_currency)
                            .and_then(|p| p.as_f64())
                            .unwrap_or(0.0);
                        
                        let change_24h_percent = price_obj.get(&format!("{}_24h_change", vs_currency))
                            .and_then(|c| c.as_f64());
                        
                        let change_24h = if let Some(change_pct) = change_24h_percent {
                            // Calculate absolute change from percentage change
                            // Formula: change = (current_price * percentage) / 100
                            Some((price * change_pct) / 100.0)
                        } else {
                            None
                        };
                        
                        let mut price_data = PriceData::new(
                            symbol.clone(),
                            price,
                            "coingecko".to_string(),
                        );
                        
                        price_data.change_24h = change_24h;
                        price_data.change_24h_percent = change_24h_percent;
                        
                        prices.push(price_data);
                    }
                }
            }
        }
        
        info!("Successfully fetched {} prices from CoinGecko", prices.len());
        Ok(prices)
    }
    
    /// Fetch prices from Binance API with 24h ticker data
    pub async fn fetch_binance_prices(&self, symbols: &[String]) -> Result<Vec<PriceData>> {
        let mut prices = Vec::new();
        info!("Fetching Binance prices for symbols: {:?}", symbols);
        
        for symbol in symbols {
            let binance_symbol = format!("{}USDT", symbol.to_uppercase());
            let url = format!("https://api.binance.com/api/v3/ticker/24hr?symbol={}", binance_symbol);
            
            info!("Fetching Binance 24hr ticker for: {} (URL: {})", binance_symbol, url);
            
            match self.fetch_binance_24hr_ticker(&url, symbol).await {
                Ok(price_data) => {
                    info!("Successfully fetched {} price: ${:.2}", price_data.symbol, price_data.price);
                    prices.push(price_data);
                },
                Err(e) => {
                    warn!("Failed to fetch Binance price for {}: {}", symbol, e);
                }
            }
        }
        
        info!("Successfully fetched {} prices from Binance", prices.len());
        Ok(prices)
    }
    
    async fn fetch_binance_24hr_ticker(&self, url: &str, original_symbol: &str) -> Result<PriceData> {
        let url = url.to_string();
        let symbol = original_symbol.to_string();
        let client = self.fetcher.client().clone();
        
        self.fetcher.retry_with_backoff(|| async {
            let response = client.get(&url).send().await?;
            
            if !response.status().is_success() {
                return Err(OracleError::ApiError(
                    format!("Binance API error: {}", response.status())
                ));
            }
            
            let ticker_data: serde_json::Value = response.json().await?;
            
            // Debug: Print the response structure
            debug!("Binance ticker response: {}", serde_json::to_string_pretty(&ticker_data).unwrap_or_default());
            
            let price: f64 = ticker_data["lastPrice"].as_str()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| {
                    OracleError::ApiError(format!(
                        "Invalid price format from Binance. Response: {}", 
                        ticker_data.get("lastPrice").map(|v| v.to_string()).unwrap_or_default()
                    ))
                })?;
                
            let price_change: f64 = ticker_data["priceChange"].as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
                
            let price_change_percent: f64 = ticker_data["priceChangePercent"].as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
                
            let volume: f64 = ticker_data["volume"].as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
                
            info!("Parsed Binance data: price={}, change={}, change%={}", price, price_change, price_change_percent);
            
            let mut price_data = PriceData::new(
                symbol.to_uppercase(),
                price,
                "binance".to_string(),
            );
            
            price_data.change_24h = Some(price_change);
            price_data.change_24h_percent = Some(price_change_percent);
            price_data.volume_24h = Some(volume);
            
            Ok(price_data)
        }).await
    }
    
    /// Get popular crypto symbols mapping
    pub fn get_symbol_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        
        // CoinGecko ID to Symbol mapping for Top 50 cryptocurrencies
        mapping.insert("bitcoin".to_string(), "BTC".to_string());
        mapping.insert("ethereum".to_string(), "ETH".to_string());
        mapping.insert("tether".to_string(), "USDT".to_string());
        mapping.insert("binancecoin".to_string(), "BNB".to_string());
        mapping.insert("solana".to_string(), "SOL".to_string());
        mapping.insert("usd-coin".to_string(), "USDC".to_string());
        mapping.insert("staked-ether".to_string(), "STETH".to_string());
        mapping.insert("ripple".to_string(), "XRP".to_string());
        mapping.insert("dogecoin".to_string(), "DOGE".to_string());
        mapping.insert("toncoin".to_string(), "TON".to_string());
        mapping.insert("cardano".to_string(), "ADA".to_string());
        mapping.insert("avalanche-2".to_string(), "AVAX".to_string());
        mapping.insert("shiba-inu".to_string(), "SHIB".to_string());
        mapping.insert("chainlink".to_string(), "LINK".to_string());
        mapping.insert("bitcoin-cash".to_string(), "BCH".to_string());
        mapping.insert("polkadot".to_string(), "DOT".to_string());
        mapping.insert("near".to_string(), "NEAR".to_string());
        mapping.insert("polygon".to_string(), "MATIC".to_string());
        mapping.insert("litecoin".to_string(), "LTC".to_string());
        mapping.insert("internet-computer".to_string(), "ICP".to_string());
        mapping.insert("dai".to_string(), "DAI".to_string());
        mapping.insert("uniswap".to_string(), "UNI".to_string());
        mapping.insert("ethereum-classic".to_string(), "ETC".to_string());
        mapping.insert("aptos".to_string(), "APT".to_string());
        mapping.insert("monero".to_string(), "XMR".to_string());
        mapping.insert("stellar".to_string(), "XLM".to_string());
        mapping.insert("okb".to_string(), "OKB".to_string());
        mapping.insert("filecoin".to_string(), "FIL".to_string());
        mapping.insert("arbitrum".to_string(), "ARB".to_string());
        mapping.insert("cosmos".to_string(), "ATOM".to_string());
        mapping.insert("hedera-hashgraph".to_string(), "HBAR".to_string());
        mapping.insert("vechain".to_string(), "VET".to_string());
        mapping.insert("blockstack".to_string(), "STX".to_string());
        mapping.insert("algorand".to_string(), "ALGO".to_string());
        mapping.insert("optimism".to_string(), "OP".to_string());
        mapping.insert("fantom".to_string(), "FTM".to_string());
        mapping.insert("the-sandbox".to_string(), "SAND".to_string());
        mapping.insert("aave".to_string(), "AAVE".to_string());
        mapping.insert("theta-token".to_string(), "THETA".to_string());
        mapping.insert("flow".to_string(), "FLOW".to_string());
        mapping.insert("axie-infinity".to_string(), "AXS".to_string());
        mapping.insert("elrond-egd-2".to_string(), "EGLD".to_string());
        mapping.insert("tezos".to_string(), "XTZ".to_string());
        mapping.insert("decentraland".to_string(), "MANA".to_string());
        mapping.insert("maker".to_string(), "MKR".to_string());
        mapping.insert("eos".to_string(), "EOS".to_string());
        mapping.insert("klay-token".to_string(), "KLAY".to_string());
        mapping.insert("neo".to_string(), "NEO".to_string());
        mapping.insert("curve-dao-token".to_string(), "CRV".to_string());
        mapping.insert("pancakeswap-token".to_string(), "CAKE".to_string());
        
        mapping
    }
    
    /// Fetch comprehensive crypto data using multiple sources
    pub async fn fetch_all_crypto_prices(&self) -> Result<Vec<PriceData>> {
        let symbols = &self.fetcher.config().crypto.symbols;
        
        if symbols.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try CoinGecko first (more comprehensive data) but with rate limit consideration
        match self.fetch_coingecko_prices(symbols).await {
            Ok(prices) if !prices.is_empty() => {
                info!("Successfully fetched prices from CoinGecko");
                return Ok(prices);
            }
            Ok(_) => {
                warn!("CoinGecko returned empty results, trying Binance fallback");
            }
            Err(e) => {
                warn!("CoinGecko failed ({}), trying Binance fallback", e);
            }
        }
        
        // Fallback to Binance
        let symbol_mapping = Self::get_symbol_mapping();
        let binance_symbols: Vec<String> = symbols.iter()
            .filter_map(|id| symbol_mapping.get(id))
            .cloned()
            .collect();
            
        if !binance_symbols.is_empty() {
            info!("Trying Binance fallback with {} symbols", binance_symbols.len());
            match self.fetch_binance_prices(&binance_symbols).await {
                Ok(prices) => {
                    info!("Successfully fetched {} prices from Binance as fallback", prices.len());
                    return Ok(prices);
                }
                Err(e) => {
                    error!("Binance fallback failed: {}", e);
                }
            }
        } else {
            warn!("No valid symbols found for Binance fallback");
        }
        
        Err(OracleError::ApiError("All crypto price sources failed".to_string()))
    }
}