use sqlx::{query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct AccountStockHoldingBalanceSnapshot {
  pub unit: String,
  pub account_id: Uuid,
  pub currency_id: Uuid,
  pub current_price: String,
}

#[tracing::instrument]
pub async fn get_account_stock_holding_balance_snapshots(
  db_client: &Pool<Postgres>,
) -> Result<Vec<AccountStockHoldingBalanceSnapshot>, String> {
  query_as!(
    AccountStockHoldingBalanceSnapshot,
    r#"
      SELECT ast.account_id as "account_id!", ast.unit, s.current_price, s.currency_id as "currency_id!"
      FROM everytrack_backend.account_stock AS ast
      JOIN everytrack_backend.stock AS s
      ON s.id = ast.stock_id
    "#,
  )
  .fetch_all(db_client)
  .await
  .map_err(|e| format!("failed to get account stock holding balance snapshots from database. {}", e))
}
