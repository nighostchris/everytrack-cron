use crate::external::db;
use dotenvy::var;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use tracing::debug;

#[tracing::instrument]
pub async fn record_exchange_rate_snapshots() -> Result<(), Box<dyn Error>> {
  // Setup database connection
  let db_client = db::client::init().await?;

  // Get value for environment variable 'EXCHANGE_RATES_API_URL'
  let exchange_rates_api_url = var("EXCHANGE_RATES_API_URL").map_err(|e| {
    format!(
      "missing config for environment variable EXCHANGE_RATES_API_URL. {}",
      e
    )
  })?;

  // Get all supported currencies from postgres database
  let supported_currencies = db::query::currency::get_all_currencies(&db_client).await?;
  debug!("got all supported currencies from database");

  // Try to fetch exchange rates API using each supported currency one by one
  for currency in supported_currencies.iter() {
    let interested_currencies = supported_currencies
      .iter()
      .filter(|&c| c.id != currency.id)
      .collect::<Vec<&db::query::currency::Currency>>();
    let response = reqwest::get(format!(
      "{exchange_rates_api_url}/{base_currency}.json",
      base_currency = currency.ticker.to_lowercase(),
    ))
    .await
    .map_err(|e| {
      format!(
        "failed to fetch exchange rates with base currency {}. {}",
        currency.ticker, e
      )
    })?;

    // Convert raw API response to consumable exchange rates json for processing
    let exchange_rate_data = response
      .json::<HashMap<String, Value>>()
      .await
      .map_err(|e| {
        format!(
          "failed to fetch exchange rates with base currency {}. {}",
          currency.ticker, e
        )
      })?;
    debug!("fetched exchange currencies API successfully. going to extract exchange rate pair");

    // Extract exchange rates pair based on target source currency
    let exchange_rate_list = exchange_rate_data
      .get(&currency.ticker.to_lowercase())
      .ok_or_else(|| {
        format!(
          "exchange rate list does not exist for base currency {}",
          currency.ticker.to_lowercase()
        )
      })?
      .as_object()
      .unwrap();
    for &target_currency in interested_currencies.iter() {
      let exchange_rate_value = exchange_rate_list
        .get(&target_currency.ticker.to_lowercase())
        .ok_or_else(|| {
          format!(
            "exchange rate value does not exist for target currency {}",
            target_currency.ticker.to_lowercase()
          )
        })?
        .as_f64()
        .unwrap();
      println!(
        "{}:{} = {}",
        currency.ticker, target_currency.ticker, exchange_rate_value
      );
    }
  }

  // End database connection
  db_client.close().await;

  Ok(())
}
