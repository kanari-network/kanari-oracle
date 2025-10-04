use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::collections::HashSet;
use std::time::Duration;
use tokio::signal;
use tokio::time;

use kanari_api::api;
use kanari_oracle::config::Config;
use kanari_oracle::oracle::Oracle;

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
        /// Configuration file path
        #[arg(short, long, default_value = "config.json")]
        config: String,
    },
    /// List all available symbols
    List {
        /// Asset type to list (crypto, stock, or all)
        #[arg(short, long, default_value = "all")]
        asset_type: String,
        /// Configuration file path
        #[arg(short, long, default_value = "config.json")]
        config: String,
    },
    /// Show price statistics
    Stats {
        /// Configuration file path
        #[arg(short, long, default_value = "config.json")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set default log level if not provided (avoid unsafe set_var)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, interval } => start_oracle_service(config, interval).await,
        Commands::Server {
            config,
            port,
            interval,
        } => start_api_server_with_updates(config, port, interval).await,
        Commands::Price {
            symbol,
            asset_type,
            config,
        } => get_single_price(symbol, asset_type, config).await,
        Commands::List { asset_type, config } => list_symbols(asset_type, config).await,
        Commands::Stats { config } => show_statistics(config).await,
    }
}

async fn start_oracle_service(config_path: String, interval: u64) -> Result<()> {
    info!("Starting Kanari Oracle Service...");

    let config = Config::from_file(&config_path)
        .await
        .context("Failed to load config")?;
    let mut oracle = Oracle::new(config)
        .await
        .context("Failed to initialize oracle")?;

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

async fn get_single_price(symbol: String, asset_type: String, config_path: String) -> Result<()> {
    let config = Config::from_file(&config_path)
        .await
        .context("Failed to load config")?;
    let oracle = Oracle::new(config)
        .await
        .context("Failed to initialize oracle")?;

    let price = match asset_type.as_str() {
        "crypto" => {
            let available: HashSet<String> = oracle
                .get_crypto_symbols()
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
            if !available.contains(&symbol.to_lowercase()) {
                error!("Symbol '{}' not configured for crypto", symbol);
                return Ok(());
            }
            oracle
                .get_crypto_price(&symbol)
                .await
                .context("Failed to fetch crypto price")?
        }
        "stock" => {
            let available: HashSet<String> = oracle
                .get_stock_symbols()
                .into_iter()
                .map(|s| s.to_uppercase())
                .collect();
            if !available.contains(&symbol.to_uppercase()) {
                error!("Symbol '{}' not configured for stock", symbol);
                return Ok(());
            }
            oracle
                .get_stock_price(&symbol)
                .await
                .context("Failed to fetch stock price")?
        }
        _ => {
            error!("Invalid asset type. Use 'crypto' or 'stock'");
            return Ok(());
        }
    };

    println!(
        "Current price for {}: ${:.2}",
        symbol.to_uppercase(),
        price.price
    );
    println!("Last updated: {}", price.timestamp);

    Ok(())
}

async fn list_symbols(asset_type: String, config_path: String) -> Result<()> {
    let config = Config::from_file(&config_path)
        .await
        .context("Failed to load config")?;
    let oracle = Oracle::new(config)
        .await
        .context("Failed to initialize oracle")?;

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

async fn show_statistics(config_path: String) -> Result<()> {
    let config = Config::from_file(&config_path)
        .await
        .context("Failed to load config")?;
    let mut oracle = Oracle::new(config)
        .await
        .context("Failed to initialize oracle")?;

    // Fetch prices to populate stats
    oracle
        .update_all_prices()
        .await
        .context("Failed to fetch prices for stats")?;

    println!("=== Oracle Statistics ===");
    let stats = oracle.get_price_statistics();

    for (key, value) in stats {
        match key.as_str() {
            "total_crypto_symbols" => {
                println!("Total Crypto Symbols: {}", value.as_u64().unwrap_or(0))
            }
            "total_stock_symbols" => {
                println!("Total Stock Symbols: {}", value.as_u64().unwrap_or(0))
            }
            "last_update" => println!("Last Update: {}", value.as_str().unwrap_or("N/A")),
            "avg_crypto_price" => println!(
                "Average Crypto Price: ${:.2}",
                value.as_f64().unwrap_or(0.0)
            ),
            "avg_stock_price" => {
                println!("Average Stock Price: ${:.2}", value.as_f64().unwrap_or(0.0))
            }
            _ => println!("{}: {:?}", key, value),
        }
    }

    Ok(())
}

async fn start_api_server_with_updates(
    config_path: String,
    port: u16,
    interval: u64,
) -> Result<()> {
    info!("Starting Kanari Oracle API Server...");

    let config = Config::from_file(&config_path)
        .await
        .context("Failed to load config")?;
    let oracle = Oracle::new(config)
        .await
        .context("Failed to initialize oracle")?;

    info!("Oracle initialized successfully");
    info!("Starting API server on port {}", port);

    // Create shared oracle for both API and background updates
    let shared_oracle = std::sync::Arc::new(tokio::sync::RwLock::new(oracle));
    let shared_oracle_clone = shared_oracle.clone();

    // Start background price updater
    let mut update_handle = tokio::spawn(async move {
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
    let mut api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server_with_shared_oracle(shared_oracle, port).await {
            error!("API server error: {}", e);
        }
    });

    // Wait for either task to stop or Ctrl+C for graceful shutdown
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping...");
        }
        _ = &mut update_handle => {
            error!("Background updater stopped unexpectedly");
        }
        _ = &mut api_handle => {
            error!("API server stopped unexpectedly");
        }
    }

    // Abort background tasks
    update_handle.abort();
    api_handle.abort();

    Ok(())
}
