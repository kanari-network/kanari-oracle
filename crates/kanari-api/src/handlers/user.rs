use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
};
use chrono::{NaiveDateTime, Utc};
use rand::rngs::OsRng;
use sqlx::Row;
use std::collections::HashMap;

use crate::api::AppState;
use crate::auth::{create_monthly_token, validate_token};
use crate::models::{
    ApiResponse, DeleteAccountRequest, LoginRequest, RegisterRequest, TokenResponse,
    UserListResponse, UserProfile,
};

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

// List all users (admin endpoint - requires valid token)
pub async fn list_users(
    Query(query): Query<HashMap<String, String>>,
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
        "SELECT id, username, email, created_at FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    {
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
    let response = UserListResponse { users, total_count };

    Ok(Json(ApiResponse::success(response)))
}

// Get current user profile
pub async fn get_user_profile(
    Query(query): Query<HashMap<String, String>>,
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
            return Ok(Json(ApiResponse::error("Token not found".to_string())));
        }
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let username: String = match owner_row.try_get("owner") {
        Ok(u) => u,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Get user details
    let user_row =
        match sqlx::query("SELECT id, username, email, created_at FROM users WHERE username = $1")
            .bind(&username)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return Ok(Json(ApiResponse::error("User not found".to_string())));
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
    Query(query): Query<HashMap<String, String>>,
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
            return Ok(Json(ApiResponse::error("Token not found".to_string())));
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
            return Ok(Json(ApiResponse::error("User not found".to_string())));
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
        return Ok(Json(ApiResponse::error("Invalid password".to_string())));
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
