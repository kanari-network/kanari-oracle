use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub price: f64,
    pub change_24h: Option<f64>,
    pub change_24h_percent: Option<f64>,
    pub volume_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl PriceData {
    pub fn new(symbol: String, price: f64, source: String) -> Self {
        Self {
            symbol,
            price,
            change_24h: None,
            change_24h_percent: None,
            volume_24h: None,
            market_cap: None,
            timestamp: Utc::now(),
            source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPriceResponse {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub total_volume: Option<f64>,
    pub market_cap: Option<f64>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPriceResponse {
    #[serde(rename = "Global Quote")]
    pub global_quote: StockQuote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    #[serde(rename = "01. symbol")]
    pub symbol: String,
    #[serde(rename = "05. price")]
    pub price: String,
    #[serde(rename = "09. change")]
    pub change: String,
    #[serde(rename = "10. change percent")]
    pub change_percent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinancePriceResponse {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceTickerResponse {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "priceChange")]
    pub price_change: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: String,
    pub symbol: String,
    pub target_price: f64,
    pub condition: AlertCondition,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    pub prices: HashMap<String, PriceData>,
    pub last_update: DateTime<Utc>,
}

impl PriceFeed {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
            last_update: Utc::now(),
        }
    }
    
    pub fn update_price(&mut self, price_data: PriceData) {
        self.prices.insert(price_data.symbol.clone(), price_data);
        self.last_update = Utc::now();
    }
    
    pub fn get_price(&self, symbol: &str) -> Option<&PriceData> {
        self.prices.get(&symbol.to_uppercase())
    }
    
    pub fn get_all_prices(&self) -> Vec<&PriceData> {
        self.prices.values().collect()
    }
}