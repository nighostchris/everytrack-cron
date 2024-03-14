use crate::external::db::client;
use crate::external::db::query::currency::{get_all_currencies, Currency};
use crate::external::db::query::exchange_rate::{
  check_existing_exchange_rate, create_new_exchange_rate, update_exchange_rate,
  CheckExistingExchangeRateParams, CreateNewExchangeRateParams, UpdateExchangeRateParams,
};
use dotenvy::var;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::error::Error;
use time::{format_description, Date, Duration, OffsetDateTime, Time};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ExchangeRateRecord {
  rate: String,
  base_currency_id: String,
  target_currency_id: String,
}

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
  // Setup postgresql database connection
  let pg_client = client::init_pg().await?;

  // Calculate the string and unix format for yesterday
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

  // Fetch and process the exchange rate pairs
  let records =
    fetch_and_process_exchange_rates(&pg_client, string_format_yesterday.as_str()).await?;
  debug!("extracted all exchange rate pairs successfully. going to insert them into database");

  // Convert exchange rate into mongodb snapshot schema
  let snapshots = records
    .into_iter()
    .map(|r| ExchangeRateSnapshot {
      rate: r.rate,
      date: yesterday.unix_timestamp(),
      base_currency_id: r.base_currency_id.clone(),
      target_currency_id: r.target_currency_id.clone(),
      _id: format!(
        "{}-{}-{}",
        r.base_currency_id, r.target_currency_id, string_format_yesterday
      ),
    })
    .collect::<Vec<ExchangeRateSnapshot>>();

  // Insert snapshots into mongodb database
  let mdb_client = client::init_mdb().await?;
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

#[tracing::instrument]
pub async fn update_latest_exchange_rates() -> Result<(), Box<dyn Error>> {
  // Setup postgresql database connection
  let pg_client = client::init_pg().await?;

  // Fetch and process the exchange rate pairs
  let records = fetch_and_process_exchange_rates(&pg_client, "latest").await?;
  debug!("extracted all exchange rate pairs successfully. going to insert them into database");

  for record in records.iter() {
    // Check if there is existing record in database already
    let is_exchange_rate_record_existed = check_existing_exchange_rate(
      &pg_client,
      CheckExistingExchangeRateParams {
        base_currency_id: Uuid::parse_str(&record.base_currency_id)?,
        target_currency_id: Uuid::parse_str(&record.target_currency_id)?,
      },
    )
    .await?;

    if is_exchange_rate_record_existed {
      debug!(
        "going to update exchange rate for pair {}:{}",
        record.base_currency_id, record.target_currency_id
      );
      update_exchange_rate(
        &pg_client,
        UpdateExchangeRateParams {
          rate: record.rate.clone(),
          base_currency_id: Uuid::parse_str(&record.base_currency_id)?,
          target_currency_id: Uuid::parse_str(&record.target_currency_id)?,
        },
      )
      .await?;
    } else {
      debug!(
        "going to create new exchange rate record for pair {}:{}",
        record.base_currency_id, record.target_currency_id
      );
      create_new_exchange_rate(
        &pg_client,
        CreateNewExchangeRateParams {
          rate: record.rate.clone(),
          base_currency_id: Uuid::parse_str(&record.base_currency_id)?,
          target_currency_id: Uuid::parse_str(&record.target_currency_id)?,
        },
      )
      .await?;
    }
  }

  // End database connection
  pg_client.close().await;

  Ok(())
}

#[tracing::instrument]
pub async fn fetch_and_process_exchange_rates(
  pg_client: &Pool<Postgres>,
  date: &str,
) -> Result<Vec<ExchangeRateRecord>, Box<dyn Error>> {
  // Get value for environment variable 'EXCHANGE_RATES_API_URL'
  let exchange_rates_api_url = var("EXCHANGE_RATES_API_URL").map_err(|e| {
    format!(
      "missing config for environment variable EXCHANGE_RATES_API_URL. {}",
      e
    )
  })?;

  // Get all supported currencies from postgres database
  let currencies = get_all_currencies(&pg_client).await?;
  debug!("got all supported currencies from database");

  // Try to fetch exchange rates API using each supported currency one by one
  let mut records: Vec<ExchangeRateRecord> = vec![];

  for currency in currencies.iter() {
    let base_currency_ticker = currency.ticker.to_lowercase();
    let interested_currencies = currencies
      .iter()
      .filter(|c| c.id != currency.id)
      .collect::<Vec<&Currency>>();
    let response = reqwest::get(format!(
      "{exchange_rates_api_url}@{date}/v1/currencies/{base_currency_ticker}.json",
    ))
    .await
    .map_err(|e| {
      format!(
        "failed to fetch exchange rates with base currency {base_currency_ticker}. {}",
        e
      )
    })?;

    // Convert raw API response to consumable exchange rates json for processing
    let exchange_rate_data = response
      .json::<HashMap<String, Value>>()
      .await
      .map_err(|e| {
        format!(
          "failed to fetch exchange rates with base currency {base_currency_ticker}. {}",
          e
        )
      })?;
    debug!("fetched exchange currencies API successfully. going to extract exchange rate pair");

    // Extract exchange rates pair based on target source currency
    let exchange_rate_list = exchange_rate_data
      .get(&base_currency_ticker)
      .ok_or_else(|| {
        format!("exchange rate list does not exist for base currency {base_currency_ticker}")
      })?
      .as_object()
      .unwrap();
    for target_currency in interested_currencies.iter() {
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
      records.push(ExchangeRateRecord {
        rate: format!("{:.8}", exchange_rate_value),
        base_currency_id: currency.id.to_string(),
        target_currency_id: target_currency.id.to_string(),
      });
    }
  }

  Ok(records)
}
