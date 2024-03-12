use sqlx::{query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct AccountBalanceSnapshot {
  id: Uuid,
  balance: String,
  currency_id: Uuid,
}

#[tracing::instrument]
pub async fn get_account_balance_snapshots(
  db_client: &Pool<Postgres>,
) -> Result<Vec<AccountBalanceSnapshot>, String> {
  query_as!(
    AccountBalanceSnapshot,
    r#"
      SELECT id, balance, currency_id as "currency_id!"
      FROM everytrack_backend.account
    "#,
  )
  .fetch_all(db_client)
  .await
  .map_err(|e| {
    format!(
      "failed to get account balance snapshots from database. {}",
      e
    )
  })
}
