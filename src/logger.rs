use std::collections::HashMap;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::field::Field;
use tracing::{span, Event, Subscriber};
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

struct CustomLayer;

#[derive(Clone, Debug, Default)]
struct JsonStorage<'a> {
  storage: HashMap<&'a str, serde_json::Value>,
}

impl<'a> JsonStorage<'a> {
  pub fn get_storage(&self) -> &HashMap<&'a str, serde_json::Value> {
    &self.storage
  }
}

pub fn init() {
  tracing_subscriber::registry()
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
      // axum logs rejections from built-in extractors are at TRACE level
      "chat_rs=debug,axum::rejection=trace,tower_http=debug".into()
    }))
    .with(CustomLayer)
    .init();
}

impl<S> Layer<S> for CustomLayer
where
  S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
  // We want to record all the key-value pairs of function parameters if there are any
  fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
    let span = ctx.span(id).expect("Span not found.");
    let mut extensions = span.extensions_mut();

    // Inherit fields from parent span if there is one
    let mut visitor = if let Some(parent_span) = span.parent() {
      // Extensions can be used for storing additional data to a span
      let mut extensions = parent_span.extensions_mut();
      extensions
        .get_mut::<JsonStorage>()
        .map(|value| value.to_owned())
        .unwrap_or_default()
    } else {
      JsonStorage::default()
    };

    // Record all the fields of current span
    attrs.values().record(&mut visitor);
    // Insert the storage into current span extensions
    extensions.insert(visitor);
  }

  fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
    let current_span = ctx.lookup_current();
    // Record all the fields of current event
    let mut event_visitor = JsonStorage::default();
    event.record(&mut event_visitor);

    // Initialize the HashMap that will store all the fields that construct the log message
    let mut output: HashMap<&str, serde_json::Value> = HashMap::new();
    // Log the event time
    output.insert(
      "timestamp",
      serde_json::Value::String(OffsetDateTime::now_utc().format(&Rfc3339).unwrap()),
    );
    // Log the log message level
    output.insert(
      "level",
      serde_json::Value::String(event.metadata().level().to_string().to_lowercase()),
    );
    // Log the location / module where the span / event happens
    output.insert(
      "target",
      serde_json::Value::String(event.metadata().target().to_string()),
    );
    // Log the custom message we typed
    event_visitor.get_storage().iter().for_each(|(key, value)| {
      output.insert(key, value.clone());
    });

    // Log the function parameters if there are some
    let mut parameters: HashMap<&str, serde_json::Value> = HashMap::new();
    if let Some(span) = current_span {
      let extensions = span.extensions();
      // Log function name
      output.insert(
        "function",
        serde_json::Value::String(span.name().to_string()),
      );
      // Get the data from extensions that store in on_new_span step
      if let Some(visitor) = extensions.get::<JsonStorage>() {
        for (key, value) in visitor.get_storage() {
          match key.to_owned() {
            "method" => output.insert(key, value.clone()),
            "uri" => output.insert("path", value.clone()),
            "version" => None,
            _ => parameters.insert(key, value.clone()),
          };
        }
      }
    }
    if parameters.len() > 0 {
      output.insert("params", serde_json::json!(parameters));
    }

    println!(
      "{}",
      serde_json::to_string_pretty(&serde_json::json!(output)).unwrap()
    );
  }
}

impl<'a> Visit for JsonStorage<'a> {
  fn record_i64(&mut self, field: &Field, value: i64) {
    self.storage.insert(field.name(), serde_json::json!(value));
  }

  fn record_f64(&mut self, field: &Field, value: f64) {
    self.storage.insert(field.name(), serde_json::json!(value));
  }

  fn record_u64(&mut self, field: &Field, value: u64) {
    self.storage.insert(field.name(), serde_json::json!(value));
  }

  fn record_bool(&mut self, field: &Field, value: bool) {
    self.storage.insert(field.name(), serde_json::json!(value));
  }

  fn record_str(&mut self, field: &Field, value: &str) {
    self.storage.insert(field.name(), serde_json::json!(value));
  }

  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    self
      .storage
      .insert(field.name(), serde_json::json!(format!("{:?}", value)));
  }

  fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
    self
      .storage
      .insert(field.name(), serde_json::json!(value.to_string()));
  }
}
