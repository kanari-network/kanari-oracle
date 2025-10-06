use serde::{Deserialize, Serialize};

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
pub struct TokenInfo {
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TokenListResponse {
    pub tokens: Vec<TokenInfo>,
}

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    // optional label to identify token on client
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct RevokeTokenRequest {
    pub token: String,
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

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    // If true, revoke other tokens for this user (keeps the current token)
    pub revoke_others: Option<bool>,
}

#[derive(Deserialize)]
pub struct ChangeEmailRequest {
    pub current_password: String,
    pub new_email: Option<String>,
}
