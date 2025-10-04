use axum::{extract::State, response::Json};

use crate::api::AppState;
use crate::models::{ApiResponse, HealthResponse};

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
