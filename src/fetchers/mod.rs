use reqwest::Client;
use std::time::Duration;
use log::warn;
use crate::errors::Result;
use crate::config::Config;

pub mod crypto;
pub mod stock;

pub use crypto::CryptoFetcher;
pub use stock::StockFetcher;

#[derive(Debug)]
pub struct PriceFetcher {
    client: Client,
    config: Config,
}

impl PriceFetcher {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.general.request_timeout))
            .build()?;
            
        Ok(Self { client, config })
    }
    

    
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    pub async fn retry_with_backoff<T, E, F, Fut>(&self, mut operation: F) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.config.general.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    warn!("Attempt {}/{} failed: {}", attempt, self.config.general.max_retries, error);
                    last_error = Some(error);
                    
                    if attempt < self.config.general.max_retries {
                        tokio::time::sleep(Duration::from_millis(
                            self.config.general.retry_delay * attempt as u64
                        )).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}