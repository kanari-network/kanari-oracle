use anyhow::anyhow;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use dotenvy;

use kanari_oracle::oracle::Oracle;

use crate::database::{create_db_pool, initialize_database, DbPool};
use crate::handlers::{
    health_check, get_price, get_all_prices, list_symbols, get_stats, 
    update_prices, register_user, login_user, list_users, 
    get_user_profile, delete_user_account
};

pub type SharedOracle = Arc<RwLock<Oracle>>;

#[derive(Clone)]
pub struct AppState {
    pub oracle: SharedOracle,
    pub db: DbPool,
}

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
pub async fn health_check(State(state): State<AppState>) -> Json<ApiResponse<HealthResponse>> {
    let oracle_lock = state.oracle.read().await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        last_update: oracle_lock.get_last_update().to_rfc3339(),
        total_symbols: oracle_lock.get_crypto_symbols().len()
            + oracle_lock.get_stock_symbols().len(),
    };

    Json(ApiResponse::success(response))
}

// Get price for a specific symbol
pub async fn get_price(
    Path((asset_type, symbol)): Path<(String, String)>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PriceResponse>>, StatusCode> {
    // Validate token
    if let Some(token) = query.get("token") {
        if !validate_token(&state.db, token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        )));
    }
    let oracle_lock = state.oracle.read().await;

    let result = match asset_type.as_str() {
        "crypto" => oracle_lock.get_crypto_price(&symbol).await,
        "stock" => oracle_lock.get_stock_price(&symbol).await,
        _ => {
            return Ok(Json(ApiResponse::error(
                "Invalid asset type. Use 'crypto' or 'stock'".to_string(),
            )));
        }
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
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PriceResponse>>>, StatusCode> {
    if let Some(token) = query.get("token") {
        if !validate_token(&state.db, token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        )));
    }
    let oracle_lock = state.oracle.read().await;

    let prices = match asset_type.as_str() {
        "crypto" => oracle_lock.get_all_crypto_prices_map(),
        "stock" => oracle_lock.get_all_stock_prices_map(),
        _ => {
            return Ok(Json(ApiResponse::error(
                "Invalid asset type. Use 'crypto' or 'stock'".to_string(),
            )));
        }
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
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Json<ApiResponse<SymbolsResponse>> {
    if let Some(token) = query.get("token") {
        // Allow list without token? enforce token
        if !validate_token(&state.db, token).await {
            return Json(ApiResponse::error("Invalid or expired token".to_string()));
        }
    } else {
        return Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        ));
    }
    let oracle_lock = state.oracle.read().await;

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
pub async fn get_stats(
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Json<ApiResponse<StatsResponse>> {
    if let Some(token) = query.get("token") {
        if !validate_token(&state.db, token).await {
            return Json(ApiResponse::error("Invalid or expired token".to_string()));
        }
    } else {
        return Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        ));
    }
    let oracle_lock = state.oracle.read().await;
    let stats = oracle_lock.get_price_statistics();

    let response = StatsResponse {
        total_crypto_symbols: stats
            .get("total_crypto_symbols")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        total_stock_symbols: stats
            .get("total_stock_symbols")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        last_update: oracle_lock.get_last_update().to_rfc3339(),
        avg_crypto_price: stats
            .get("avg_crypto_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        avg_stock_price: stats
            .get("avg_stock_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        uptime_seconds: 0, // TODO: Implement uptime tracking
    };

    Json(ApiResponse::success(response))
}

// Force update prices
pub async fn update_prices(
    Path(asset_type): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    if let Some(token) = query.get("token") {
        if !validate_token(&state.db, token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        )));
    }
    let mut oracle_lock = state.oracle.write().await;

    let result = match asset_type.as_str() {
        "crypto" => oracle_lock.update_crypto_prices().await,
        "stock" => oracle_lock.update_stock_prices().await,
        "all" => oracle_lock.update_all_prices().await,
        _ => {
            return Ok(Json(ApiResponse::error(
                "Invalid asset type. Use 'crypto', 'stock', or 'all'".to_string(),
            )));
        }
    };

    match result {
        Ok(count) => Ok(Json(ApiResponse::success(format!(
            "Updated {} price feeds",
            count
        )))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
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

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub owner_email: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserProfile>,
    pub total_count: i32,
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

// Register a new user and return an API token
pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<TokenResponse>>, StatusCode> {
    // hash password using Argon2id with default params
    let argon2 = Argon2::default();
    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let hashed = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(ph) => ph.to_string(),
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // insert user
    let res = sqlx::query("INSERT INTO users (username, password_hash, email) VALUES ($1, $2, $3)")
        .bind(&payload.username)
        .bind(&hashed)
        .bind(payload.owner_email.as_deref())
        .execute(&state.db)
        .await;

    if let Err(e) = res {
        return Ok(Json(ApiResponse::error(e.to_string())));
    }

    // create token
    match create_monthly_token(&state.db, &payload.username).await {
        Ok(token) => {
            // fetch expiry
            let row = sqlx::query("SELECT expires_at FROM api_tokens WHERE token = $1")
                .bind(&token)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires_naive: NaiveDateTime = row
                .try_get("expires_at")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires = chrono::DateTime::<Utc>::from_naive_utc_and_offset(expires_naive, Utc);
            Ok(Json(ApiResponse::success(TokenResponse {
                token,
                expires_at: expires.to_rfc3339(),
            })))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Login: validate credentials and return existing/new token
pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<TokenResponse>>, StatusCode> {
    let row = sqlx::query("SELECT password_hash FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let hash_val: String = match row {
        Some(r) => r
            .try_get("password_hash")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        None => {
            return Ok(Json(ApiResponse::error(
                "Invalid username or password".to_string(),
            )));
        }
    };

    // verify Argon2 password
    let parsed_hash =
        PasswordHash::new(&hash_val).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(Json(ApiResponse::error(
            "Invalid username or password".to_string(),
        )));
    }

    // Create and return a new token
    match create_monthly_token(&state.db, &payload.username).await {
        Ok(token) => {
            let row = sqlx::query("SELECT expires_at FROM api_tokens WHERE token = $1")
                .bind(&token)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires_naive: NaiveDateTime = row
                .try_get("expires_at")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires = chrono::DateTime::<Utc>::from_naive_utc_and_offset(expires_naive, Utc);
            Ok(Json(ApiResponse::success(TokenResponse {
                token,
                expires_at: expires.to_rfc3339(),
            })))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// (old non-DB start_api_server_with_shared_oracle removed)

// Validate a token exists and is not expired
async fn validate_token(db: &DbPool, token: &str) -> bool {
    match sqlx::query("SELECT expires_at FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            let expires_naive: Result<NaiveDateTime, _> = row.try_get("expires_at");
            match expires_naive {
                Ok(exp_naive) => {
                    let exp = chrono::DateTime::<Utc>::from_naive_utc_and_offset(exp_naive, Utc);
                    exp > Utc::now()
                }
                Err(_) => false,
            }
        }
        _ => false,
    }
}

// Create a monthly token for an owner (simple helper)
pub async fn create_monthly_token(db: &DbPool, owner: &str) -> anyhow::Result<String> {
    let token = Uuid::new_v4().to_string();
    let expires = Utc::now() + Duration::days(30);

    sqlx::query("INSERT INTO api_tokens (token, owner, expires_at) VALUES ($1, $2, $3)")
        .bind(&token)
        .bind(owner)
        .bind(expires.naive_utc())
        .execute(db)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(token)
}

// List all users (admin endpoint - requires valid token)
pub async fn list_users(
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserListResponse>>, StatusCode> {
    if let Some(token) = query.get("token") {
        if !validate_token(&state.db, token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing token query parameter".to_string(),
        )));
    }

    let rows = match sqlx::query(
        "SELECT id, username, email, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await {
        Ok(rows) => rows,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let mut users = Vec::new();
    for row in &rows {
        let id: i32 = row.try_get("id").unwrap_or(0);
        let username: String = row.try_get("username").unwrap_or_default();
        let email: Option<String> = row.try_get("email").ok();
        let created_at_naive: NaiveDateTime = row.try_get("created_at").unwrap_or_default();
        let created_at = chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_at_naive, Utc);

        users.push(UserProfile {
            id,
            username,
            email,
            created_at: created_at.to_rfc3339(),
        });
    }

    let total_count = users.len() as i32;
    let response = UserListResponse {
        users,
        total_count,
    };

    Ok(Json(ApiResponse::success(response)))
}

// Get current user profile
pub async fn get_user_profile(
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    let token = match query.get("token") {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing token query parameter".to_string(),
            )));
        }
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error(
            "Invalid or expired token".to_string(),
        )));
    }

    // Get username from token
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Ok(Json(ApiResponse::error(
                "Token not found".to_string(),
            )));
        }
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let username: String = match owner_row.try_get("owner") {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Get user details
    let user_row = match sqlx::query(
        "SELECT id, username, email, created_at FROM users WHERE username = $1"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Ok(Json(ApiResponse::error(
                "User not found".to_string(),
            )));
        }
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let id: i32 = user_row.try_get("id").unwrap_or(0);
    let email: Option<String> = user_row.try_get("email").ok();
    let created_at_naive: NaiveDateTime = user_row.try_get("created_at").unwrap_or_default();
    let created_at = chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_at_naive, Utc);

    let profile = UserProfile {
        id,
        username,
        email,
        created_at: created_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(profile)))
}

// Delete user account (requires password confirmation)
pub async fn delete_user_account(
    Query(query): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let token = match query.get("token") {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing token query parameter".to_string(),
            )));
        }
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error(
            "Invalid or expired token".to_string(),
        )));
    }

    // Get username from token
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Ok(Json(ApiResponse::error(
                "Token not found".to_string(),
            )));
        }
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let username: String = match owner_row.try_get("owner") {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Verify password
    let user_row = match sqlx::query("SELECT password_hash FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Ok(Json(ApiResponse::error(
                "User not found".to_string(),
            )));
        }
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let hash_val: String = match user_row.try_get("password_hash") {
        Ok(h) => h,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Verify password
    let parsed_hash = match PasswordHash::new(&hash_val) {
        Ok(h) => h,
        Err(_) => {
            return Ok(Json(ApiResponse::error(
                "Invalid password hash".to_string(),
            )));
        }
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(Json(ApiResponse::error(
            "Invalid password".to_string(),
        )));
    }

    // Delete user (this will cascade delete tokens due to foreign key)
    match sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&state.db)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Account deleted successfully".to_string(),
        ))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Initialize database tables if they don't exist
async fn initialize_database(pool: &DbPool) -> anyhow::Result<()> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create api_tokens table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_tokens (
            id SERIAL PRIMARY KEY,
            token VARCHAR(255) UNIQUE NOT NULL,
            owner VARCHAR(255) NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            FOREIGN KEY (owner) REFERENCES users(username) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    log::info!("Database tables created/verified: users, api_tokens");
    Ok(())
}

pub async fn start_api_server_with_shared_oracle(
    shared_oracle: SharedOracle,
    port: u16,
) -> anyhow::Result<()> {
    // Load .env file (if present) so DATABASE_URL and other env vars are available
    dotenvy::dotenv().ok();
    // Build DB pool from DATABASE_URL env var
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL must be set"))?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

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

    axum::serve(listener, app).await?;

    Ok(())
}
