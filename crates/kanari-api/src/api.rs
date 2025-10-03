use axum::{
    Router,
    routing::{get, post},
};
use dotenvy;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use kanari_oracle::oracle::Oracle;

use crate::database::{DbPool, create_db_pool, initialize_database};
use crate::handlers::{
    delete_user_account, get_all_prices, get_price, get_stats, get_user_profile, health_check,
    list_symbols, list_users, login_user, register_user, update_prices,
};

pub type SharedOracle = Arc<RwLock<Oracle>>;

#[derive(Clone)]
pub struct AppState {
    pub oracle: SharedOracle,
    pub db: DbPool,
}

pub fn create_router(oracle: SharedOracle, db: DbPool) -> Router {
    let state = AppState { oracle, db };
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
        // User endpoints
        .route("/users/register", post(register_user))
        .route("/users/login", post(login_user))
        .route("/users/list", get(list_users))
        .route("/users/profile", get(get_user_profile))
        .route("/users/delete", post(delete_user_account))
        // Add state
        .with_state(state)
        // Add middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

pub async fn start_api_server_with_shared_oracle(
    shared_oracle: SharedOracle,
    port: u16,
) -> anyhow::Result<()> {
    // Load .env file (if present) so DATABASE_URL and other env vars are available
    dotenvy::dotenv().ok();

    // Build DB pool from DATABASE_URL env var
    let pool = create_db_pool().await?;

    // Initialize database tables
    initialize_database(&pool).await?;
    log::info!("Database tables initialized successfully");

    let app = create_router(shared_oracle, pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    log::info!("🚀 API server starting on http://0.0.0.0:{}", port);
    log::info!("📚 API Documentation:");
    log::info!("  GET  /health                     - Health check");
    log::info!("  GET  /price/:type/:symbol        - Get specific price (crypto/btc, stock/aapl)");
    log::info!("  GET  /prices/:type               - Get all prices for type (crypto, stock)");
    log::info!("  GET  /symbols?asset_type=type    - List available symbols");
    log::info!("  GET  /stats                      - Oracle statistics");
    log::info!("  POST /update/:type               - Force update prices (crypto, stock, all)");
    log::info!("  POST /users/register             - Register new user");
    log::info!("  POST /users/login                - User login");
    log::info!("  GET  /users/list                 - List all users");
    log::info!("  GET  /users/profile              - Get user profile");
    log::info!("  POST /users/delete               - Delete user account");

    axum::serve(listener, app).await?;

    Ok(())
}
