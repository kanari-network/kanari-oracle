use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use chrono::{NaiveDateTime, Utc};
use rand::rngs::OsRng;
use sqlx::Row;

use crate::api::AppState;
use crate::auth::{create_monthly_token, validate_token};
use crate::models::{
    ApiResponse, ChangePasswordRequest, DeleteAccountRequest, LoginRequest, RegisterRequest,
    TokenResponse, UserListResponse, UserProfile,
};

use crate::models::{TokenInfo, TokenListResponse, CreateTokenRequest};

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

// List API tokens for the authenticated user
pub async fn list_user_tokens(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<TokenListResponse>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => return Ok(Json(ApiResponse::error("Missing Authorization header".to_string()))),
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error("Invalid or expired token".to_string())));
    }

    // Get owner
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        _ => return Ok(Json(ApiResponse::error("Token not found".to_string()))),
    };

    let owner: String = match owner_row.try_get("owner") {
        Ok(o) => o,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid token owner".to_string()))),
    };

    let rows = match sqlx::query("SELECT token, expires_at, created_at FROM api_tokens WHERE owner = $1 ORDER BY created_at DESC")
        .bind(&owner)
        .fetch_all(&state.db)
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let mut tokens = Vec::new();
    for row in &rows {
        let tok: String = row.try_get("token").unwrap_or_default();
        let expires_naive: NaiveDateTime = row.try_get("expires_at").unwrap_or_default();
        let created_naive: NaiveDateTime = row.try_get("created_at").unwrap_or_default();
        let expires = chrono::DateTime::<Utc>::from_naive_utc_and_offset(expires_naive, Utc);
        let created = chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_naive, Utc);

        tokens.push(TokenInfo {
            token: tok,
            expires_at: expires.to_rfc3339(),
            created_at: created.to_rfc3339(),
        });
    }

    Ok(Json(ApiResponse::success(TokenListResponse { tokens })))
}

// Create a new API token for the authenticated user
pub async fn create_user_token(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(_payload): Json<CreateTokenRequest>,
) -> Result<Json<ApiResponse<TokenResponse>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => return Ok(Json(ApiResponse::error("Missing Authorization header".to_string()))),
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error("Invalid or expired token".to_string())));
    }

    // Find owner
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        _ => return Ok(Json(ApiResponse::error("Token not found".to_string()))),
    };

    let owner: String = match owner_row.try_get("owner") {
        Ok(o) => o,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid token owner".to_string()))),
    };

    match create_monthly_token(&state.db, &owner).await {
        Ok(new_token) => {
            let row = sqlx::query("SELECT expires_at FROM api_tokens WHERE token = $1")
                .bind(&new_token)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires_naive: NaiveDateTime = row
                .try_get("expires_at")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let expires = chrono::DateTime::<Utc>::from_naive_utc_and_offset(expires_naive, Utc);
            Ok(Json(ApiResponse::success(TokenResponse {
                token: new_token,
                expires_at: expires.to_rfc3339(),
            })))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Delete (revoke) a specific token for the authenticated user's account
pub async fn delete_user_token(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<crate::models::RevokeTokenRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => return Ok(Json(ApiResponse::error("Missing Authorization header".to_string()))),
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error("Invalid or expired token".to_string())));
    }

    // Ensure the requester owns the token they are deleting
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        _ => return Ok(Json(ApiResponse::error("Token not found".to_string()))),
    };

    let owner: String = match owner_row.try_get("owner") {
        Ok(o) => o,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid token owner".to_string()))),
    };

    // Verify the payload token belongs to the same owner
    let target_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(&payload.token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(Json(ApiResponse::error("Token to delete not found".to_string()))),
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    let target_owner: String = match target_row.try_get("owner") {
        Ok(o) => o,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid token owner".to_string()))),
    };

    if target_owner != owner {
        return Ok(Json(ApiResponse::error("Cannot delete token for another user".to_string())));
    }

    match sqlx::query("DELETE FROM api_tokens WHERE token = $1")
        .bind(&payload.token)
        .execute(&state.db)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success("Token revoked".to_string()))),
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
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserListResponse>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing Authorization header".to_string(),
            )));
        }
    };

    if !validate_token(&state.db, token).await {
        return Ok(Json(ApiResponse::error(
            "Invalid or expired token".to_string(),
        )));
    }

    // Check whether the token owner is an admin. If the users table doesn't have an is_admin
    // column, default to denying access (safe-by-default). This requires a users.is_admin boolean.
    let owner_row = match sqlx::query("SELECT owner FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        _ => {
            return Ok(Json(ApiResponse::error("Token not found".to_string())));
        }
    };

    let owner: String = match owner_row.try_get("owner") {
        Ok(o) => o,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid token owner".to_string()))),
    };

    // Check is_admin flag on users table. If column missing, this query will error; handle gracefully.
    let is_admin =
        match sqlx::query_scalar::<_, bool>("SELECT is_admin FROM users WHERE username = $1")
            .bind(&owner)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some(flag)) => flag,
            Ok(None) => false,
            Err(_) => false,
        };

    if !is_admin {
        return Ok(Json(ApiResponse::error(
            "Admin privileges required".to_string(),
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
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing Authorization header".to_string(),
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
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing Authorization header".to_string(),
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

// Change user password (requires current password confirmation)
pub async fn change_user_password(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    let token = match token {
        Some(t) => t,
        None => {
            return Ok(Json(ApiResponse::error(
                "Missing Authorization header".to_string(),
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

    // Verify current password
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

    let current_hash_val: String = match user_row.try_get("password_hash") {
        Ok(h) => h,
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Verify current password
    let parsed_current_hash = match PasswordHash::new(&current_hash_val) {
        Ok(h) => h,
        Err(_) => {
            return Ok(Json(ApiResponse::error(
                "Invalid current password hash".to_string(),
            )));
        }
    };

    if Argon2::default()
        .verify_password(payload.current_password.as_bytes(), &parsed_current_hash)
        .is_err()
    {
        return Ok(Json(ApiResponse::error(
            "Current password is incorrect".to_string(),
        )));
    }

    // Hash new password using Argon2id with default params
    let argon2 = Argon2::default();
    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let new_hashed = match argon2.hash_password(payload.new_password.as_bytes(), &salt) {
        Ok(ph) => ph.to_string(),
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };

    // Update password in database
    match sqlx::query("UPDATE users SET password_hash = $1 WHERE username = $2")
        .bind(&new_hashed)
        .bind(&username)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            // Optionally revoke other tokens for this user
            if payload.revoke_others.unwrap_or(false) {
                // Delete all tokens for owner except the current token
                let _ = sqlx::query("DELETE FROM api_tokens WHERE owner = $1 AND token <> $2")
                    .bind(&username)
                    .bind(token)
                    .execute(&state.db)
                    .await;
            }

            Ok(Json(ApiResponse::success(
                "Password changed successfully".to_string(),
            )))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}
