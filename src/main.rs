mod config;
mod external;
mod logger;
mod server;

use config::load_env_vars;
use external::db;

#[tokio::main]
async fn main() {
  // Load environment variables
  load_env_vars();
  // Initialize logger
  logger::init();
  // Create a new database client
  let db_client = db::client::init().await;
  // Initialize web server
  server::init(db_client.clone()).await;
}
