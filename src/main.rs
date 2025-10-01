use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, error};
use std::time::Duration;
use tokio::time;

mod oracle;
mod fetchers;
mod models;
mod config;
mod errors;

use oracle::Oracle;
use config::Config;

#[derive(Parser)]
#[command(name = "kanari-oracle")]
#[command(about = "A real-time Oracle system for cryptocurrency and stock prices")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the oracle service
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "config.json")]
        config: String,
        /// Update interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },
    /// Get current price for a symbol
    Price {
        /// Symbol to get price for (e.g., BTC, AAPL)
        symbol: String,
        /// Asset type (crypto or stock)
        #[arg(short, long, default_value = "crypto")]
        asset_type: String,
    },
    /// List all available symbols
    List {
        /// Asset type to list (crypto, stock, or all)
        #[arg(short, long, default_value = "all")]
        asset_type: String,
    },
    /// Show price statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { config, interval } => {
            start_oracle_service(config, interval).await
        }
        Commands::Price { symbol, asset_type } => {
            get_single_price(symbol, asset_type).await
        }
        Commands::List { asset_type } => {
            list_symbols(asset_type).await
        }
        Commands::Stats => {
            show_statistics().await
        }
    }
}

async fn start_oracle_service(config_path: String, interval: u64) -> Result<()> {
    info!("Starting Kanari Oracle Service...");
    
    let config = Config::from_file(&config_path).await?;
    let mut oracle = Oracle::new(config).await?;
    
    info!("Oracle initialized successfully");
    info!("Update interval: {} seconds", interval);
    
    let mut update_interval = time::interval(Duration::from_secs(interval));
    
    loop {
        update_interval.tick().await;
        
        match oracle.update_all_prices().await {
            Ok(count) => info!("Updated {} price feeds", count),
            Err(e) => error!("Failed to update prices: {}", e),
        }
        
        // Print current prices
        oracle.print_current_prices();
    }
}

async fn get_single_price(symbol: String, asset_type: String) -> Result<()> {
    let config = Config::default();
    let oracle = Oracle::new(config).await?;
    
    let price = match asset_type.as_str() {
        "crypto" => oracle.get_crypto_price(&symbol).await?,
        "stock" => oracle.get_stock_price(&symbol).await?,
        _ => {
            error!("Invalid asset type. Use 'crypto' or 'stock'");
            return Ok(());
        }
    };
    
    println!("Current price for {}: ${:.2}", symbol.to_uppercase(), price.price);
    println!("Last updated: {}", price.timestamp);
    
    Ok(())
}

async fn list_symbols(asset_type: String) -> Result<()> {
    let config = Config::default();
    let oracle = Oracle::new(config).await?;
    
    match asset_type.as_str() {
        "crypto" => {
            println!("Available Cryptocurrencies:");
            for symbol in oracle.get_crypto_symbols() {
                println!("  {}", symbol);
            }
        }
        "stock" => {
            println!("Available Stocks:");
            for symbol in oracle.get_stock_symbols() {
                println!("  {}", symbol);
            }
        }
        "all" => {
            println!("Available Cryptocurrencies:");
            for symbol in oracle.get_crypto_symbols() {
                println!("  {} (crypto)", symbol);
            }
            println!("\nAvailable Stocks:");
            for symbol in oracle.get_stock_symbols() {
                println!("  {} (stock)", symbol);
            }
        }
        _ => {
            error!("Invalid asset type. Use 'crypto', 'stock', or 'all'");
        }
    }
    
    Ok(())
}

async fn show_statistics() -> Result<()> {
    let config = Config::default();
    let oracle = Oracle::new(config).await?;
    
    println!("=== Oracle Statistics ===");
    let stats = oracle.get_price_statistics();
    
    for (key, value) in stats {
        match key.as_str() {
            "total_crypto_symbols" => println!("Total Crypto Symbols: {}", value.as_u64().unwrap_or(0)),
            "total_stock_symbols" => println!("Total Stock Symbols: {}", value.as_u64().unwrap_or(0)),
            "last_update" => println!("Last Update: {}", value.as_str().unwrap_or("N/A")),
            "avg_crypto_price" => println!("Average Crypto Price: ${:.2}", value.as_f64().unwrap_or(0.0)),
            "avg_stock_price" => println!("Average Stock Price: ${:.2}", value.as_f64().unwrap_or(0.0)),
            _ => println!("{}: {:?}", key, value),
        }
    }
    
    Ok(())
}