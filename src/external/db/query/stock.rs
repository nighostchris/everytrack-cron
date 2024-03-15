use sqlx::{query, query_as, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct Stock {
  pub id: Uuid,
  pub name: String,
  pub ticker: String,
  pub country_id: Uuid,
  pub currency_id: Uuid,
  pub current_price: String,
}

#[derive(Debug)]
pub struct UpdateStockCurrentPriceParams {
  pub id: Uuid,
  pub current_price: String,
}

#[tracing::instrument]
pub async fn get_all_stocks_by_country_id(pg_client: &Pool<Postgres>, country_id: &str) -> Result<Vec<Stock>, String> {
  query_as!(
    Stock,
    r#"
      SELECT s.id, country_id as "country_id!", currency_id as "currency_id!", s.name, ticker, current_price
      FROM everytrack_backend.stock AS s
      INNER JOIN everytrack_backend.country AS c
      ON s.country_id = c.id
      WHERE s.country_id = $1
    "#,
    Uuid::parse_str(country_id).unwrap(),
  )
  .fetch_all(pg_client)
  .await
  .map_err(|e| format!("failed to get all stocks by country id from postgresql database. {}", e))
}

#[tracing::instrument]
pub async fn update_stock_current_price(pg_client: &Pool<Postgres>, params: UpdateStockCurrentPriceParams) -> Result<(), String> {
  let rows_affected = query!(
    r#"
      UPDATE everytrack_backend.stock
      SET current_price = $1 WHERE id = $2
    "#,
    params.current_price,
    params.id,
  )
  .execute(pg_client)
  .await
  .map_err(|e| format!("failed to update stock current price in postgresql database. {}", e))?
  .rows_affected();

  if rows_affected.ge(&0) {
    Ok(())
  } else {
    Err("unexpected error occured when updating stock current price in postgresql database".to_string())
  }
}
