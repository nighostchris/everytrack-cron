// mod balance;
mod exchange_rate;

use std::error::Error;
use std::future::Future;
use time::{format_description, OffsetDateTime};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{debug, error};

// Cronjob Syntax
// sec   min   hour   day of month   month   day of week   year
// *     *     *      *              *       *             *

static CRONJOB_NAMES: [&str; 1] = ["record_exchange_rate_snapshots"];

#[tracing::instrument]
pub async fn init() {
  let scheduler = JobScheduler::new()
    .await
    .expect("Failed to initialize cronjob scheduler");
  debug!("initialized cronjob scheduler");

  // Create cronjobs

  let cronjobs = vec![
    // Record exchange rate snapshots every day at 00:00
    create_cronjob(
      CRONJOB_NAMES[0],
      "0 * * * * * *",
      exchange_rate::record_exchange_rate_snapshots,
    ),
  ];
  debug!("going to add jobs to cronjob scheduler");

  // Adding jobs to scheduler
  for (index, cronjob) in cronjobs.into_iter().flatten().enumerate() {
    debug!(
      "added cronjob {} into cronjob scheduler",
      CRONJOB_NAMES[index]
    );
    scheduler.add(cronjob).await.unwrap_or_else(|e| {
      panic!(
        "Failed to add cronjob of recording exchange rate snapshots. {}",
        e
      )
    });
  }
  debug!("added all cronjobs into cronjob scheduler");

  // Start the scheduler
  match scheduler.start().await {
    Ok(_) => debug!("cronjob scheduler started"),
    Err(e) => panic!("Failed to start the scheduler. {}", e),
  };
}

fn create_cronjob<F, Fut>(
  name: &'static str,
  schedule: &'static str,
  task: F,
) -> Result<Job, JobSchedulerError>
where
  F: Fn() -> Fut + Clone + Send + Sync + 'static,
  Fut: Future<Output = Result<(), Box<dyn Error>>> + Send,
{
  Job::new_async(schedule, move |uuid, mut l| {
    let task = task.clone();
    Box::pin(async move {
      debug!(
        "start executing cronjob {name} at {}",
        OffsetDateTime::now_utc()
          .format(
            &format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z").unwrap()
          )
          .unwrap()
      );
      if let Err(e) = task().await {
        error!("{}", e);
      }
      match l.next_tick_for_job(uuid).await {
        Ok(Some(timestamp)) => debug!("next scheduled time for cronjob {name} is {timestamp:?}"),
        _ => debug!("cannot get next scheduled time for cronjob {name}"),
      };
    })
  })
}
