// mod balance;
mod exchange_rate;

use crate::external::db;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error};

// Cronjob Syntax
// sec   min   hour   day of month   month   day of week   year
// *     *     *      *              *       *             *

#[tracing::instrument]
pub async fn init() {
  let scheduler = JobScheduler::new()
    .await
    .expect("Failed to initialize cronjob scheduler");
  debug!("initialized cronjob scheduler");

  // Record exchange rate snapshots every day at 00:00
  let record_exchange_rate_snapshots_job = Job::new_async("0 * * * * * *", |uuid, mut l| {
    Box::pin(async move {
      debug!("started recording exchange rate snapshots");
      let db_client = match db::client::init().await {
        Ok(client) => client,
        Err(e) => {
          error!("{}", e);
          return;
        }
      };
      if let Err(e) = exchange_rate::record_exchange_rate_snapshots(&db_client).await {
        error!("{}", e);
      };
      match l.next_tick_for_job(uuid).await {
        Ok(Some(timestamp)) => debug!(
          "will be recording exchange rate snapshots at {:?} again",
          timestamp
        ),
        _ => debug!("cannot get next scheduled time for recording exchange rate snapshots"),
      };
    })
  })
  .unwrap();

  debug!("going to add jobs to cronjob scheduler");

  // Adding jobs to scheduler
  scheduler
    .add(record_exchange_rate_snapshots_job)
    .await
    .unwrap_or_else(|e| {
      panic!(
        "Failed to add cronjob of recording exchange rate snapshots. {}",
        e
      )
    });
  debug!("inserted all jobs into cronjob scheduler");

  // Start the scheduler
  match scheduler.start().await {
    Ok(_) => debug!("cronjob scheduler started"),
    Err(e) => panic!("Failed to start the scheduler. {}", e),
  };
}
