use dotenvy::var;
use mongodb::options::ClientOptions;
use mongodb::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::info;

#[tracing::instrument]
pub async fn init_pg() -> Result<Pool<Postgres>, String> {
  // Try to get the environment variable 'DATABASE' that stores the postgresql database connection url
  let db_conn_url = var("DATABASE").expect("PostgreSQL database connection url is invalid.");
  info!("initializing postgresql database client");

  let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(db_conn_url.as_str())
    .await
    .map_err(|e| format!("Cannot initiate postgresql database connection. {}", e));
  info!("database client initialized");

  return pool;
}

#[tracing::instrument]
pub async fn init_mdb() -> Result<Client, String> {
  // Try to get the environment variable 'MONGODB' that stores the mongodb database connection url
  let db_conn_url = var("MONGODB").expect("MongoDB database connection url is invalid.");
  info!("initializing mongodb database client");

  let client_options = ClientOptions::parse(db_conn_url)
    .await
    .map_err(|e| format!("Cannot initialize mongodb client options. {}", e))?;
  let client = Client::with_options(client_options)
    .map_err(|e| format!("Cannot initialize mongodb database connection. {}", e));

  return client;
}
