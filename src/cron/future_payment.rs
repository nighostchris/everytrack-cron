use crate::external::db::client;
use crate::external::db::query::account::{get_account_balance_by_id, update_account_balance, UpdateAccountBalanceParams};
use crate::external::db::query::future_payment::{
  delete_future_payment, get_all_future_payments, update_future_payment_schedule, UpdateFuturePaymentScheduleParams,
};
use crate::external::db::query::transaction::{create_new_transaction, CreateNewTransactionParams};
use crate::utils::format_timestamp;
use rust_decimal::Decimal;
use std::error::Error;
use std::str::FromStr;
use time::{Date, Duration, OffsetDateTime, Time};
use tracing::debug;

#[tracing::instrument]
pub async fn monitor_future_payments() -> Result<(), Box<dyn Error>> {
  // Setup postgresql database connection
  let pg_client = client::init_pg().await?;

  // Get all future payments of all users in database
  let future_payments = get_all_future_payments(&pg_client).await?;
  debug!("got all future payments from postgresql database");

  for future_payment in future_payments.iter() {
    let today = OffsetDateTime::now_utc();
    let start_of_today = OffsetDateTime::new_utc(
      Date::from_calendar_date(today.year(), today.month(), today.day()).unwrap(),
      Time::from_hms_nano(0, 0, 0, 0).unwrap(),
    );

    if future_payment.scheduled_at.gt(&today) {
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
    let original_account_balance = get_account_balance_by_id(&pg_client, future_payment.account_id).await?;
    let original_account_balance_decimal =
      Decimal::from_str(&original_account_balance).map_err(|e| format!("failed to parse original account balance into decimal. {}", e))?;
    let payment_amount_decimal =
      Decimal::from_str(&future_payment.amount).map_err(|e| format!("failed to parse future payment amount into decimal. {}", e))?;

    // Calculate the final account balance after spending / receiving the payment amount
    let mut final_account_balance = original_account_balance_decimal.clone();
    if future_payment.income {
      final_account_balance += payment_amount_decimal;
    } else {
      final_account_balance -= payment_amount_decimal;
    }
    debug!(
      "going to update balance for account {} from {} to {:.2}",
      future_payment.account_id,
      original_account_balance,
      final_account_balance.to_string()
    );

    // Update account balance after spending / receiving scheduled payment
    update_account_balance(
      &pg_client,
      UpdateAccountBalanceParams {
        id: future_payment.account_id,
        balance: format!("{:.2}", final_account_balance.to_string()),
      },
    )
    .await?;

    // Create a new transaction record according to the payment details
    create_new_transaction(
      &pg_client,
      CreateNewTransactionParams {
        executed_at: start_of_today,
        income: future_payment.income,
        name: future_payment.name.clone(),
        client_id: future_payment.client_id,
        account_id: future_payment.account_id,
        amount: future_payment.amount.clone(),
        currency_id: future_payment.currency_id,
        remarks: future_payment.remarks.clone(),
        category: future_payment.category.clone(),
      },
    )
    .await?;

    // Update next schedule date according to frequency if payment is on rolling basis
    if future_payment.rolling {
      let mut next_schedule_date = future_payment.scheduled_at.clone();
      let frequency = future_payment.frequency.unwrap();
      let days_to_add = frequency / 86400;

      if days_to_add >= 29 {
        let months_to_add = days_to_add / 30;
        let mut final_month = next_schedule_date.month().nth_next(months_to_add.try_into().unwrap());
        if months_to_add == 0 {
          final_month = final_month.next();
        }
        next_schedule_date = next_schedule_date.replace_month(final_month).unwrap();
      } else {
        next_schedule_date = next_schedule_date.replace_date(next_schedule_date.date().checked_add(Duration::days(days_to_add)).unwrap());
      }
      debug!(
        "going to update next schedule for future payment {}({}) to {}",
        future_payment.name,
        future_payment.id,
        format_timestamp(next_schedule_date)?
      );

      update_future_payment_schedule(
        &pg_client,
        UpdateFuturePaymentScheduleParams {
          id: future_payment.id,
          scheduled_at: next_schedule_date,
        },
      )
      .await?;
    } else {
      // Delete future payment as it is not rolling, i.e. one-off payment
      delete_future_payment(&pg_client, future_payment.id).await?;
    }

    debug!("finished processing future payment {}({})", future_payment.name, future_payment.id,);
  }

  // End database connection
  pg_client.close().await;

  Ok(())
}
