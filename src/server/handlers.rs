use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct BaseResponse {
  pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub success: bool,
  pub error: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
  pub success: bool,
  pub result: T,
}

// Handler function for path '/'
#[tracing::instrument]
pub async fn health_check_handler() -> impl IntoResponse {
  info!("received request");
  (StatusCode::OK, Json(BaseResponse { success: true }))
}
