use sqlx::{query, query_as, Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct FuturePayment {
  pub id: Uuid,
  pub client_id: Uuid,
  pub account_id: Uuid,
  pub currency_id: Uuid,
  pub name: String,
  pub amount: String,
  pub income: bool,
  pub rolling: bool,
  pub category: String,
  pub frequency: Option<i64>,
  pub remarks: Option<String>,
  pub scheduled_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct UpdateFuturePaymentScheduleParams {
  pub id: Uuid,
  pub scheduled_at: OffsetDateTime,
}

#[tracing::instrument]
pub async fn get_all_future_payments(pg_client: &Pool<Postgres>) -> Result<Vec<FuturePayment>, String> {
  query_as!(
    FuturePayment,
    r#"
      SELECT id, client_id as "client_id!", account_id as "account_id!", currency_id as "currency_id!", name, amount, income, rolling, category, frequency, remarks, scheduled_at
      FROM everytrack_backend.future_payment
    "#,
  )
  .fetch_all(pg_client)
  .await
  .map_err(|e| {
    format!(
      "failed to get all future payments from database. {}",
      e
    )
  })
}

#[tracing::instrument]
pub async fn update_future_payment_schedule(pg_client: &Pool<Postgres>, params: UpdateFuturePaymentScheduleParams) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      UPDATE everytrack_backend.future_payment
      SET scheduled_at = $1
      WHERE id = $2
    "#,
    params.scheduled_at,
    params.id
  )
  .execute(pg_client)
  .await
  .map_err(|e| format!("failed to update future payment schedule in postgresql database. {}", e))?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when updating future payment schedule in postgresql database".to_string())
  }
}

#[tracing::instrument]
pub async fn delete_future_payment(pg_client: &Pool<Postgres>, id: Uuid) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      DELETE FROM everytrack_backend.future_payment WHERE id = $1
    "#,
    id,
  )
  .execute(pg_client)
  .await
  .map_err(|e| format!("failed to delete future payment in postgresql database. {}", e))?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when deleting future payment in postgresql database".to_string())
  }
}
