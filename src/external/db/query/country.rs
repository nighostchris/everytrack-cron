use sqlx::{query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct Country {
  pub id: Uuid,
  pub name: String,
  pub code: String,
}

#[tracing::instrument]
pub async fn get_country_by_code(pg_client: &Pool<Postgres>, code: &str) -> Result<Country, String> {
  query_as!(
    Country,
    r#"
      SELECT id, name, code FROM everytrack_backend.country
      WHERE code = $1
    "#,
    code,
  )
  .fetch_one(pg_client)
  .await
  .map_err(|e| format!("failed to get country by code from postgresql database. {}", e))
}
