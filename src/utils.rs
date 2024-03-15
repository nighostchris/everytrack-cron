use time::{format_description, OffsetDateTime};

pub fn format_timestamp(timestamp: OffsetDateTime) -> Result<String, String> {
  timestamp
    .format(
      &format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
        .map_err(|e| format!("failed to construct format description. {}", e))?,
    )
    .map_err(|e| format!("failed to format timestamp. {}", e))
}
