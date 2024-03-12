use dotenvy::var;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::info;

#[tracing::instrument]
pub async fn init() -> Result<Pool<Postgres>, String> {
  // Try to get the environment variable 'DATABASE' that stores the database connection url
  let db_conn_url = var("DATABASE").expect("Database connection url is invalid.");
  info!("initializing database client");

  let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(db_conn_url.as_str())
    .await
    .map_err(|e| format!("Cannot initiate database connection. {}", e));
  info!("database client initialized");

  return pool;
}
