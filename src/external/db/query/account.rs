use sqlx::{query, query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct AccountBalanceSnapshot {
  pub id: Uuid,
  pub balance: String,
  pub currency_id: Uuid,
}

#[tracing::instrument]
pub async fn get_account_balance_snapshots(pg_client: &Pool<Postgres>) -> Result<Vec<AccountBalanceSnapshot>, String> {
  query_as!(
    AccountBalanceSnapshot,
    r#"
      SELECT id, balance, currency_id as "currency_id!"
      FROM everytrack_backend.account
    "#,
  )
  .fetch_all(pg_client)
  .await
  .map_err(|e| format!("failed to get account balance snapshots from database. {}", e))
}

#[tracing::instrument]
pub async fn get_account_balance_by_id(pg_client: &Pool<Postgres>, id: &str) -> Result<String, String> {
  let raw = query!(
    r#"
      SELECT balance FROM everytrack_backend.account WHERE id = $1
    "#,
    Uuid::parse_str(id).unwrap(),
  )
  .fetch_one(pg_client)
  .await
  .map_err(|e| format!("failed to get account balance from database. {}", e))?;

  Ok(raw.balance)
}
