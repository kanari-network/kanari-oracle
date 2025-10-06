use anyhow::anyhow;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub type DbPool = PgPool;

// Initialize database tables if they don't exist
pub async fn initialize_database(pool: &DbPool) -> anyhow::Result<()> {
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
            expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
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

pub async fn create_db_pool() -> anyhow::Result<DbPool> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL must be set"))?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    Ok(pool)
}
