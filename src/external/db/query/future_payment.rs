use sqlx::{query_as, Pool, Postgres};
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
