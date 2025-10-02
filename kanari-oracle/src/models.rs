use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String, // เก็บรูปแบบดั้งเดิม (แต่ key ใน HashMap เป็น lowercase)
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

impl StockQuote {
    /// Convert StockQuote to PriceData
    
    #[allow(dead_code)]
    pub fn to_price_data(&self) -> Option<PriceData> {
        let price = self.price.trim().parse::<f64>().ok()?;
        let change = self.change.trim().parse::<f64>().ok()?;
        let change_percent = self.change_percent
            .trim()
            .trim_end_matches('%')
            .parse::<f64>()
            .ok()?;

        Some(PriceData {
            symbol: self.symbol.to_lowercase(), // Use lowercase for consistency
            price,
            change_24h: Some(change),
            change_24h_percent: Some(change_percent),
            volume_24h: None,
            market_cap: None,
            timestamp: Utc::now(),
            source: "alphavantage".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    pub prices: HashMap<String, PriceData>, // key = symbol.to_lowercase()
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
        let key = price_data.symbol.to_lowercase();
        self.prices.insert(key, price_data);
        self.last_update = Utc::now();
    }
    
    pub fn get_price(&self, symbol: &str) -> Option<&PriceData> {
        self.prices.get(&symbol.to_lowercase())
    }
    
    pub fn get_all_prices(&self) -> Vec<&PriceData> {
        self.prices.values().collect()
    }
    
    pub fn get_prices_map(&self) -> &HashMap<String, PriceData> {
        &self.prices
    }
}