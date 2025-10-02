use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{info, error};
use std::time::Duration;
use tokio::time;



use kanari_oracle::oracle::Oracle;
use kanari_oracle::config::Config;
use kanari_api::api;

#[derive(Parser)]
#[command(name = "kanari")]
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
    /// Start the HTTP API server
    Server {
        /// Configuration file path
        #[arg(short, long, default_value = "config.json")]
        config: String,
        /// Port to run the API server on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Update interval in seconds for background updates
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
        Commands::Server { config, port, interval } => {
            start_api_server_with_updates(config, port, interval).await
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

async fn start_api_server_with_updates(config_path: String, port: u16, interval: u64) -> Result<()> {
    info!("Starting Kanari Oracle API Server...");
    
    let config = Config::from_file(&config_path).await?;
    let oracle = Oracle::new(config).await?;
    
    info!("Oracle initialized successfully");
    info!("Starting API server on port {}", port);
    
    // Create shared oracle for both API and background updates
    let shared_oracle = std::sync::Arc::new(tokio::sync::RwLock::new(oracle));
    let shared_oracle_clone = shared_oracle.clone();
    
    // Start background price updater
    let update_handle = tokio::spawn(async move {
        let mut update_interval = time::interval(Duration::from_secs(interval));
        
        loop {
            update_interval.tick().await;
            
            let mut oracle_lock = shared_oracle_clone.write().await;
            match oracle_lock.update_all_prices().await {
                Ok(count) => info!("Background update: Updated {} price feeds", count),
                Err(e) => error!("Background update failed: {}", e),
            }
            oracle_lock.print_current_prices();
        }
    });
    
    // Start API server with shared oracle
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server_with_shared_oracle(shared_oracle, port).await {
            error!("API server error: {}", e);
        }
    });
    
    // Wait for both tasks
    tokio::select! {
        _ = update_handle => {
            error!("Background updater stopped unexpectedly");
        }
        _ = api_handle => {
            error!("API server stopped unexpectedly");
        }
    }
    
    Ok(())
}