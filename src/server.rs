mod handlers;

use axum::routing::get;
use axum::Router;
use dotenvy::var;
use handlers::health_check_handler;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
// use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

// #[derive(Debug)]
// pub struct ServerState {
//   db: Pool<Postgres>,
// }

// Initialize an axum web server instance
#[tracing::instrument]
pub async fn init(pg_client: Pool<Postgres>) {
  // let server_state = Arc::new(ServerState { db: db_client });
  // https://stackoverflow.com/questions/74302133/how-to-log-and-filter-requests-with-axum-tokio
  let service = ServiceBuilder::new().layer(TraceLayer::new_for_http());
  // Define the routes for web server
  let app = Router::new()
    .route("/", get(health_check_handler))
    // .nest("/api/v1", api_version_one_routes)
    .layer(service)
    // .with_state(server_state)
    .into_make_service();

  // Try to get the environment variables 'WEB_SERVER_HOST' and 'WEB_SERVER_PORT' that define the public exposure details of web server
  let web_server_host = var("WEB_SERVER_HOST").unwrap_or_else(|e| panic!("Missing config for environment variable WEB_SERVER_HOST. {}", e));
  let web_server_port = var("WEB_SERVER_PORT").unwrap_or_else(|e| panic!("Missing config for environment variable WEB_SERVER_PORT. {}", e));
  info!("{}", format!("web server is listening at {}:{}", web_server_host, web_server_port));

  // Finally start up server and serve the endpoints
  axum::serve(
    TcpListener::bind(&format!("{}:{}", web_server_host, web_server_port).parse::<SocketAddr>().unwrap())
      .await
      .unwrap(),
    app,
  )
  .await
  .unwrap()
}
