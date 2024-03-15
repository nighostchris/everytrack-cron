use crate::external::db::client;
use crate::external::db::query::account::get_account_balance_by_id;
use crate::external::db::query::future_payment::get_all_future_payments;
use crate::utils::format_timestamp;
use std::error::Error;
use tracing::debug;

#[tracing::instrument]
pub async fn monitor_future_payments() -> Result<(), Box<dyn Error>> {
  // Setup postgresql database connection
  let pg_client = client::init_pg().await?;

  // Get all future payments of all users in database
  let future_payments = get_all_future_payments(&pg_client).await?;
  debug!("got all future payments from postgresql database");

  for future_payment in future_payments.iter() {
    if future_payment.scheduled_at.gt(&time::OffsetDateTime::now_utc()) {
      debug!(
        "future payment {}({}) will not be processed now as the next schedule is {}",
        future_payment.name,
        future_payment.id,
        format_timestamp(future_payment.scheduled_at)?
      );
      break;
    }

    // The scheduled date for future payment has fallen behind current timestamp
    // So will process the payment
    debug!(
      "going to process future payment {}({}) of amount {} for account {}",
      future_payment.name, future_payment.id, future_payment.amount, future_payment.account_id
    );
    let account_id = future_payment.account_id.to_string();
    let original_account_balance = get_account_balance_by_id(&pg_client, &account_id).await?;
  }

  // End database connection
  pg_client.close().await;

  Ok(())
}
