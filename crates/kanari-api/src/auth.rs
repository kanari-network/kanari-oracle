use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use uuid::Uuid;
use std::collections::HashMap;

use crate::database::DbPool;

// Validate a token exists and is not expired
pub async fn validate_token(db: &DbPool, token: &str) -> bool {
    match sqlx::query("SELECT expires_at FROM api_tokens WHERE token = $1")
        .bind(token)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            // ✅ อ่านเป็น DateTime<Utc> โดยตรง - ชัดเจนและปลอดภัย
            match row.try_get::<DateTime<Utc>, _>("expires_at") {
                Ok(exp) => exp > Utc::now(),
                Err(_) => false,
            }
        }
        _ => false,
    }
}

// Extract token from Authorization header or query parameter
pub fn extract_token_from_request(
    headers: &axum::http::HeaderMap,
    query: &HashMap<String, String>,
) -> Option<String> {
    // Try Authorization header first (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // Fallback to query parameter
    query.get("token").cloned()
}

// Create a monthly token for an owner (simple helper)
pub async fn create_monthly_token(db: &DbPool, owner: &str) -> anyhow::Result<String> {
    let token = Uuid::new_v4().to_string();
    let expires: DateTime<Utc> = Utc::now() + Duration::days(30);

    sqlx::query("INSERT INTO api_tokens (token, owner, expires_at) VALUES ($1, $2, $3)")
        .bind(&token)
        .bind(owner)
        .bind(expires) // ✅ ส่ง DateTime<Utc> โดยตรง - sqlx จัดการ timezone อัตโนมัติ
        .execute(db)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(token)
}
