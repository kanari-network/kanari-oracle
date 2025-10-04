use anyhow::anyhow;
use chrono::{Duration, NaiveDateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::database::DbPool;

// Validate a token exists and is not expired
pub async fn validate_token(db: &DbPool, token: &str) -> bool {
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
