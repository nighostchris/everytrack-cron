mod config;
mod cron;
mod external;
mod logger;
mod server;

#[tokio::main]
async fn main() {
  // Load environment variables
  config::load_env_vars();
  // Initialize logger
  logger::init();
  // Create a new database client
  let db_client = external::db::client::init().await;
  // Setup cronjobs
  cron::record_exchange_rate_snapshots(&db_client).await;
  // Initialize web server
  // server::init(db_client.clone()).await;
}
