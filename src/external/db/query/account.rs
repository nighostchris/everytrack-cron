use sqlx::{query, query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct AccountBalanceSnapshot {
  pub id: Uuid,
  pub balance: String,
  pub currency_id: Uuid,
}

#[derive(Debug)]
pub struct UpdateAccountBalanceParams {
  pub id: Uuid,
  pub balance: String,
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
  .map_err(|e| format!("failed to get account balance snapshots from postgresql database. {}", e))
}

#[tracing::instrument]
pub async fn get_account_balance_by_id(pg_client: &Pool<Postgres>, id: Uuid) -> Result<String, String> {
  let raw = query!(
    r#"
      SELECT balance FROM everytrack_backend.account WHERE id = $1
    "#,
    id,
  )
  .fetch_one(pg_client)
  .await
  .map_err(|e| format!("failed to get account balance from postgresql database. {}", e))?;

  Ok(raw.balance)
}

#[tracing::instrument]
pub async fn update_account_balance(pg_client: &Pool<Postgres>, params: UpdateAccountBalanceParams) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      UPDATE everytrack_backend.account SET balance = $1 WHERE id = $2
    "#,
    params.balance,
    params.id,
  )
  .execute(pg_client)
  .await
  .map_err(|e| format!("failed to update account balance in postgresql database. {}", e))?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when updating account balance in postgresql database".to_string())
  }
}
