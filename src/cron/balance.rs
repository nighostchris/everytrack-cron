use sqlx::{Pool, Postgres};

#[tracing::instrument]
pub async fn record_account_balance_snapshots(db_client: &Pool<Postgres>) {
  //
  // let account_balance_snapshots =
  //   db::query::account::get_account_balance_snapshots(&db_client).await;
  // println!("{account_balance_snapshots:?}");
  // let account_stock_holding_balance_snapshots =
  //   db::query::account_stock::get_account_stock_holding_balance_snapshots(&db_client).await;
  // println!("{account_stock_holding_balance_snapshots:?}");
}
