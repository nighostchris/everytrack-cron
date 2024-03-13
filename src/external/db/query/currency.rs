use sqlx::{query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct Currency {
  pub id: Uuid,
  pub ticker: String,
  pub symbol: String,
}

#[tracing::instrument]
pub async fn get_all_currencies(db_client: &Pool<Postgres>) -> Result<Vec<Currency>, String> {
  query_as!(
    Currency,
    r#"
      SELECT id, ticker, symbol
      FROM everytrack_backend.currency
    "#,
  )
  .fetch_all(db_client)
  .await
  .map_err(|e| {
    format!(
      "failed to get all supported currencies from database. {}",
      e
    )
  })
}
