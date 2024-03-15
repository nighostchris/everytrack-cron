use sqlx::{query, Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct CreateNewTransactionParams {
  pub name: String,
  pub income: bool,
  pub amount: String,
  pub client_id: Uuid,
  pub category: String,
  pub account_id: Uuid,
  pub currency_id: Uuid,
  pub remarks: Option<String>,
  pub executed_at: OffsetDateTime,
}

#[tracing::instrument]
pub async fn create_new_transaction(pg_client: &Pool<Postgres>, params: CreateNewTransactionParams) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      INSERT INTO everytrack_backend.transaction (client_id, account_id, currency_id, name, category, amount, income, remarks, executed_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#,
    params.client_id,
    params.account_id,
    params.currency_id,
    params.name,
    params.category,
    params.amount,
    params.income,
    params.remarks,
    params.executed_at,
  )
  .execute(pg_client)
  .await
  .map_err(|e| format!("failed to create new transaction in postgresql database. {}", e))?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when creating new transaction in postgresql database".to_string())
  }
}
