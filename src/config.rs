use dotenvy::dotenv;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

// TODO: add validation to config for environment variables
pub fn load_env_vars() {
  dotenv().expect("Failed to load environment variables from '.env' file");
  println!(
    "{}",
    serde_json::to_string_pretty(&serde_json::json!({
      "function": "load_env_vars",
      "level": "info",
      "message": "environment variables successfully loaded",
      "target": "everytrack_cron::config",
      "timestamp": OffsetDateTime::now_utc().format(&Rfc3339).unwrap()
    }))
    .unwrap()
  );
}
