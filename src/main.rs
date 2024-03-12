mod config;
mod external;
mod logger;
mod server;

use config::load_env_vars;
// use external::db;

#[tokio::main]
async fn main() {
  // Load environment variables
  load_env_vars();
  // Initialize logger
  logger::init();
  // Create a new database client
  // let db_client = db::client::init().await;
  // Initialize web server
  // server::init(db_client.clone()).await;
  // let account_balance_snapshots =
  //   db::query::account::get_account_balance_snapshots(&db_client).await;
  // println!("{account_balance_snapshots:?}");
  // let account_stock_holding_balance_snapshots =
  //   db::query::account_stock::get_account_stock_holding_balance_snapshots(&db_client).await;
  // println!("{account_stock_holding_balance_snapshots:?}");
}
