use crate::external::db;
use dotenvy::var;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use time::{format_description, Date, Duration, OffsetDateTime, Time};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
struct ExchangeRateSnapshot {
  _id: String,
  date: i64,
  rate: String,
  base_currency_id: String,
  target_currency_id: String,
}

#[tracing::instrument]
pub async fn record_exchange_rate_snapshots() -> Result<(), Box<dyn Error>> {
  // Setup database connection
  let pg_client = db::client::init_pg().await?;
  let mdb_client = db::client::init_mdb().await?;

  // Get value for environment variable 'EXCHANGE_RATES_API_URL'
  let exchange_rates_api_url = var("EXCHANGE_RATES_API_URL").map_err(|e| {
    format!(
      "missing config for environment variable EXCHANGE_RATES_API_URL. {}",
      e
    )
  })?;

  let raw_yesterday = OffsetDateTime::now_utc()
    .checked_sub(Duration::days(1))
    .unwrap();
  let yesterday = OffsetDateTime::new_utc(
    Date::from_calendar_date(
      raw_yesterday.year(),
      raw_yesterday.month(),
      raw_yesterday.day(),
    )
    .unwrap(),
    Time::from_hms_nano(0, 0, 0, 0).unwrap(),
  );
  // YYYY-MM-DD format of yesterday
  let string_format_yesterday = yesterday
    .format(&format_description::parse("[year]-[month]-[day]").unwrap())
    .unwrap();
  let unix_format_yesterday = yesterday.unix_timestamp();

  // Get all supported currencies from postgres database
  let supported_currencies = db::query::currency::get_all_currencies(&pg_client).await?;
  debug!("got all supported currencies from database");

  // Try to fetch exchange rates API using each supported currency one by one
  let mut snapshots: Vec<ExchangeRateSnapshot> = vec![];
  for currency in supported_currencies.iter() {
    let interested_currencies = supported_currencies
      .iter()
      .filter(|&c| c.id != currency.id)
      .collect::<Vec<&db::query::currency::Currency>>();
    let response = reqwest::get(format!(
      "{exchange_rates_api_url}@{string_format_yesterday}/v1/currencies/{base_currency}.json",
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
      snapshots.push(ExchangeRateSnapshot {
        _id: format!(
          "{}-{}-{}",
          currency.id, target_currency.id, string_format_yesterday
        ),
        date: unix_format_yesterday,
        rate: format!("{:.8}", exchange_rate_value),
        base_currency_id: currency.id.to_string(),
        target_currency_id: target_currency.id.to_string(),
      });
    }
  }
  debug!("extracted all exchange rate pairs successfully. going to insert them into database");

  // Insert snapshots into mongodb database
  let mdb_snapshots_db = mdb_client.database("snapshots");
  let collection = mdb_snapshots_db.collection::<ExchangeRateSnapshot>("exchange_rate_snapshots");
  collection
    .insert_many(snapshots, None)
    .await
    .map_err(|e| format!("failed to insert snapshots into mongodb database. {}", e))?;
  debug!(
    "successfully inserted all exchange rate snapshots of {string_format_yesterday} into database"
  );

  // End database connection
  pg_client.close().await;

  Ok(())
}
