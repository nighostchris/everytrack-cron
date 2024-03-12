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
  let db_client = external::db::client::init()
    .await
    .unwrap_or_else(|e| panic!("{}", e));
  // Setup cronjobs
  cron::init().await;
  // Initialize web server
  server::init(db_client).await;
}
