use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use kanari_oracle::oracle::Oracle;

pub type SharedOracle = Arc<RwLock<Oracle>>;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price: f64,
    pub timestamp: String,
    pub asset_type: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub last_update: String,
    pub total_symbols: usize,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_crypto_symbols: usize,
    pub total_stock_symbols: usize,
    pub last_update: String,
    pub avg_crypto_price: f64,
    pub avg_stock_price: f64,
    pub uptime_seconds: i64,
}



#[derive(Deserialize)]
pub struct ListQuery {
    pub asset_type: Option<String>,
}

#[derive(Serialize)]
pub struct SymbolsResponse {
    pub crypto: Vec<String>,
    pub stocks: Vec<String>,
}

// Health check endpoint
pub async fn health_check(State(oracle): State<SharedOracle>) -> Json<ApiResponse<HealthResponse>> {
    let oracle_lock = oracle.read().await;
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        last_update: oracle_lock.get_last_update().to_rfc3339(),
        total_symbols: oracle_lock.get_crypto_symbols().len() + oracle_lock.get_stock_symbols().len(),
    };
    
    Json(ApiResponse::success(response))
}

// Get price for a specific symbol
pub async fn get_price(
    Path((asset_type, symbol)): Path<(String, String)>,
    State(oracle): State<SharedOracle>,
) -> Result<Json<ApiResponse<PriceResponse>>, StatusCode> {
    let oracle_lock = oracle.read().await;
    
    let result = match asset_type.as_str() {
        "crypto" => oracle_lock.get_crypto_price(&symbol).await,
        "stock" => oracle_lock.get_stock_price(&symbol).await,
        _ => return Ok(Json(ApiResponse::error("Invalid asset type. Use 'crypto' or 'stock'".to_string()))),
    };
    
    match result {
        Ok(price_data) => {
            let response = PriceResponse {
                symbol: symbol.to_uppercase(),
                price: price_data.price,
                timestamp: price_data.timestamp.to_rfc3339(),
                asset_type: asset_type.clone(),
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Get all prices for an asset type
pub async fn get_all_prices(
    Path(asset_type): Path<String>,
    State(oracle): State<SharedOracle>,
) -> Result<Json<ApiResponse<Vec<PriceResponse>>>, StatusCode> {
    let oracle_lock = oracle.read().await;
    
    let prices = match asset_type.as_str() {
        "crypto" => oracle_lock.get_all_crypto_prices_map(),
        "stock" => oracle_lock.get_all_stock_prices_map(),
        _ => return Ok(Json(ApiResponse::error("Invalid asset type. Use 'crypto' or 'stock'".to_string()))),
    };
    
    log::info!("API: Found {} {} prices", prices.len(), asset_type);
    
    let response: Vec<PriceResponse> = prices
        .iter()
        .map(|(symbol, price_data)| PriceResponse {
            symbol: symbol.clone(),
            price: price_data.price,
            timestamp: price_data.timestamp.to_rfc3339(),
            asset_type: asset_type.clone(),
        })
        .collect();
    
    Ok(Json(ApiResponse::success(response)))
}

// List available symbols
pub async fn list_symbols(
    Query(params): Query<ListQuery>,
    State(oracle): State<SharedOracle>,
) -> Json<ApiResponse<SymbolsResponse>> {
    let oracle_lock = oracle.read().await;
    
    let crypto_symbols = oracle_lock.get_crypto_symbols();
    let stock_symbols = oracle_lock.get_stock_symbols();
    
    let response = match params.asset_type.as_deref() {
        Some("crypto") => SymbolsResponse {
            crypto: crypto_symbols,
            stocks: vec![],
        },
        Some("stock") => SymbolsResponse {
            crypto: vec![],
            stocks: stock_symbols,
        },
        _ => SymbolsResponse {
            crypto: crypto_symbols,
            stocks: stock_symbols,
        },
    };
    
    Json(ApiResponse::success(response))
}

// Get oracle statistics
pub async fn get_stats(State(oracle): State<SharedOracle>) -> Json<ApiResponse<StatsResponse>> {
    let oracle_lock = oracle.read().await;
    let stats = oracle_lock.get_price_statistics();
    
    let response = StatsResponse {
        total_crypto_symbols: stats.get("total_crypto_symbols")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        total_stock_symbols: stats.get("total_stock_symbols")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        last_update: oracle_lock.get_last_update().to_rfc3339(),
        avg_crypto_price: stats.get("avg_crypto_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        avg_stock_price: stats.get("avg_stock_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        uptime_seconds: 0, // TODO: Implement uptime tracking
    };
    
    Json(ApiResponse::success(response))
}

// Force update prices
pub async fn update_prices(
    Path(asset_type): Path<String>,
    State(oracle): State<SharedOracle>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut oracle_lock = oracle.write().await;
    
    let result = match asset_type.as_str() {
        "crypto" => oracle_lock.update_crypto_prices().await,
        "stock" => oracle_lock.update_stock_prices().await,
        "all" => oracle_lock.update_all_prices().await,
        _ => return Ok(Json(ApiResponse::error("Invalid asset type. Use 'crypto', 'stock', or 'all'".to_string()))),
    };
    
    match result {
        Ok(count) => Ok(Json(ApiResponse::success(format!("Updated {} price feeds", count)))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub fn create_router(oracle: SharedOracle) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Price endpoints
        .route("/price/{asset_type}/{symbol}", get(get_price))
        .route("/prices/{asset_type}", get(get_all_prices))
        
        // Symbols
        .route("/symbols", get(list_symbols))
        
        // Statistics
        .route("/stats", get(get_stats))
        
        // Update endpoints
        .route("/update/{asset_type}", post(update_prices))
        
        // Add state
        .with_state(oracle)
        
        // Add middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

pub async fn start_api_server_with_shared_oracle(shared_oracle: SharedOracle, port: u16) -> anyhow::Result<()> {
    let app = create_router(shared_oracle);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    log::info!("🚀 API server starting on http://0.0.0.0:{}", port);
    log::info!("📚 API Documentation:");
    log::info!("  GET  /health                     - Health check");
    log::info!("  GET  /price/:type/:symbol        - Get specific price (crypto/btc, stock/aapl)");
    log::info!("  GET  /prices/:type               - Get all prices for type (crypto, stock)");
    log::info!("  GET  /symbols?asset_type=type    - List available symbols");
    log::info!("  GET  /stats                      - Oracle statistics");
    log::info!("  POST /update/:type               - Force update prices (crypto, stock, all)");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}