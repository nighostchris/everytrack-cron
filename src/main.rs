mod config;
mod cron;
mod external;
mod logger;
mod server;
mod utils;

use dotenvy::var;

#[tokio::main]
async fn main() {
  // Load environment variables
  let environment = var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
  if environment == "development" {
    config::load_env_vars();
  }
  // Initialize logger
  logger::init();
  // Create a new database client
  let pg_client = external::db::client::init_pg().await.unwrap_or_else(|e| panic!("{}", e));
  // Setup cronjobs
  cron::init().await;
  // Initialize web server
  server::init(pg_client).await;
}
