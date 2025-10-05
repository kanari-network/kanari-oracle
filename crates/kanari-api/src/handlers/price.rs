use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use std::collections::HashMap;

use crate::api::AppState;
use crate::auth::{validate_token, extract_token_from_request};
use crate::models::{ApiResponse, ListQuery, PriceResponse, StatsResponse, SymbolsResponse};

// Get price for a specific symbol
pub async fn get_price(
    Path((asset_type, symbol)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PriceResponse>>, StatusCode> {
    // Validate token from header or query parameter
    let token = extract_token_from_request(&headers, &query);
    
    if let Some(token) = token {
        if !validate_token(&state.db, &token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing authentication token".to_string(),
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
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PriceResponse>>>, StatusCode> {
    // Validate token from header or query parameter
    let token = extract_token_from_request(&headers, &query);
    
    if let Some(token) = token {
        if !validate_token(&state.db, &token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing authentication token".to_string(),
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
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Json<ApiResponse<SymbolsResponse>> {
    // Validate token from header or query parameter
    let token = extract_token_from_request(&headers, &query);
    
    if let Some(token) = token {
        if !validate_token(&state.db, &token).await {
            return Json(ApiResponse::error("Invalid or expired token".to_string()));
        }
    } else {
        return Json(ApiResponse::error(
            "Missing authentication token".to_string(),
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
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Json<ApiResponse<StatsResponse>> {
    // Validate token from header or query parameter
    let token = extract_token_from_request(&headers, &query);
    
    if let Some(token) = token {
        if !validate_token(&state.db, &token).await {
            return Json(ApiResponse::error("Invalid or expired token".to_string()));
        }
    } else {
        return Json(ApiResponse::error(
            "Missing authentication token".to_string(),
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
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Validate token from header or query parameter
    let token = extract_token_from_request(&headers, &query);
    
    if let Some(token) = token {
        if !validate_token(&state.db, &token).await {
            return Ok(Json(ApiResponse::error(
                "Invalid or expired token".to_string(),
            )));
        }
    } else {
        return Ok(Json(ApiResponse::error(
            "Missing authentication token".to_string(),
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
