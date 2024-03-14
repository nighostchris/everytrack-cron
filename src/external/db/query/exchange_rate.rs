use sqlx::{query, query_scalar, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct CheckExistingExchangeRateParams {
  pub base_currency_id: Uuid,
  pub target_currency_id: Uuid,
}

#[derive(Debug)]
pub struct CreateNewExchangeRateParams {
  pub rate: String,
  pub base_currency_id: Uuid,
  pub target_currency_id: Uuid,
}

#[derive(Debug)]
pub struct UpdateExchangeRateParams {
  pub rate: String,
  pub base_currency_id: Uuid,
  pub target_currency_id: Uuid,
}

#[tracing::instrument]
pub async fn check_existing_exchange_rate(
  pg_client: &Pool<Postgres>,
  params: CheckExistingExchangeRateParams,
) -> Result<bool, String> {
  let is_exchange_rate_record_exists = query_scalar!(
    r#"
      SELECT EXISTS(
        SELECT 1 FROM everytrack_backend.exchange_rate
        WHERE base_currency_id = $1 AND target_currency_id = $2
      )
    "#,
    params.base_currency_id,
    params.target_currency_id,
  )
  .fetch_one(pg_client)
  .await
  .map_err(|e| {
    format!(
      "failed to check if exchange rate record exists in postgresql database. {}",
      e
    )
  });

  is_exchange_rate_record_exists.and_then(|r| {
    r.ok_or("unexpected error occured when checking if exchange rate record exists".to_string())
  })
}

#[tracing::instrument]
pub async fn create_new_exchange_rate(
  pg_client: &Pool<Postgres>,
  params: CreateNewExchangeRateParams,
) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      INSERT INTO everytrack_backend.exchange_rate (base_currency_id, target_currency_id, rate)
      VALUES ($1, $2, $3)
    "#,
    params.base_currency_id,
    params.target_currency_id,
    params.rate,
  )
  .execute(pg_client)
  .await
  .map_err(|e| {
    format!(
      "failed to create new exchange rate record in postgresql database. {}",
      e
    )
  })?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err(
      "unexpected error occured when creating new exchange rate record in postgresql database"
        .to_string(),
    )
  }
}

#[tracing::instrument]
pub async fn update_exchange_rate(
  pg_client: &Pool<Postgres>,
  params: UpdateExchangeRateParams,
) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      UPDATE everytrack_backend.exchange_rate
      SET rate = $1
      WHERE base_currency_id = $2 AND target_currency_id = $3
    "#,
    params.rate,
    params.base_currency_id,
    params.target_currency_id,
  )
  .execute(pg_client)
  .await
  .map_err(|e| {
    format!(
      "failed to update exchange rate in postgresql database. {}",
      e
    )
  })?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when updating exchange rate in postgresql database".to_string())
  }
}
