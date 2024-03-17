use crate::external::db::client;
use crate::external::db::query::country::get_country_by_code;
use crate::external::db::query::stock::{get_all_stocks_by_country_id, update_stock_current_price, UpdateStockCurrentPriceParams};
use std::error::Error;
use tracing::debug;
use yahoo_finance_api::YahooConnector;

#[tracing::instrument]
pub async fn update_latest_us_stock_prices() -> Result<(), Box<dyn Error>> {
  update_latest_stock_prices("US").await
}

#[tracing::instrument]
pub async fn update_latest_uk_stock_prices() -> Result<(), Box<dyn Error>> {
  update_latest_stock_prices("UK").await
}

#[tracing::instrument]
pub async fn update_latest_stock_prices(country_code: &str) -> Result<(), Box<dyn Error>> {
  // Setup postgresql database connection
  let pg_client = client::init_pg().await?;

  // Setup yahoo finance api client
  let yahoo_finance_api_client = YahooConnector::new();

  // Get US country id from database
  let country = get_country_by_code(&pg_client, country_code).await?;
  debug!("got us country id from postgresql database");

  // Get all supported US stocks in database
  let supported_stocks = get_all_stocks_by_country_id(&pg_client, &country.id.to_string()).await?;
  debug!("got all supported stocks from postgresql database");

  for stock in supported_stocks.iter() {
    debug!("going to get latest price quote for stock {}", stock.ticker);

    let quote = yahoo_finance_api_client
      .get_latest_quotes(&stock.ticker, "1d")
      .await
      .map_err(|e| format!("failed to get latest quote for {}. {}", stock.ticker, e))?
      .last_quote()
      .map_err(|e| format!("failed to extract last quote from latest quote of {}. {}", stock.ticker, e))?;

    // Update current price of the target stock in database
    debug!("going to update latest price for stock {}", stock.ticker);
    update_stock_current_price(
      &pg_client,
      UpdateStockCurrentPriceParams {
        id: stock.id,
        current_price: format!("{:.2}", quote.close),
      },
    )
    .await?;
    debug!("updated latest price for stock {}", stock.ticker);
  }

  // End database connection
  pg_client.close().await;

  Ok(())
}
