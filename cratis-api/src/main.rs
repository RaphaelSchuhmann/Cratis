#[allow(unused_imports)]
use crate::handler::{authentication::{authenticate_middleware, register}, health_check::health_check};
use cratis_core::{config::{get_config_api, load_config, TEMP_API_CONFIG_PATH}};
use axum::{Router, routing::post, routing::get, middleware};
use polodb_core::Database;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Arc;

// This is for the test endpoint only:
// use http::StatusCode;

mod handler;

// Database:
pub static DB: Lazy<Arc<Database>> = Lazy::new(|| { Arc::new(Database::open_path(PathBuf::from(get_config_api().settings.db.clone())).expect("Failed to open DB")) });

#[tokio::main]
async fn main() {
    // Load config
    load_config(TEMP_API_CONFIG_PATH, true);

    // Router
    // let auth_routes = Router::new()
    //     // Put any routes that need authentication here
    //     // .route("/test", get(test))
    //     .route_layer(middleware::from_fn(authenticate_middleware));

    let public_routes = Router::new()
        .route("/register", post(register))
        .route("/ping", get(health_check));

    let app = Router::new()
        .merge(public_routes);
        // .merge(auth_routes);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", get_config_api().settings.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// This is a temporary test endpoint to test the authentication system
// Remove this endpoint when it is no longer needed
// pub async fn test() -> StatusCode {
//     StatusCode::IM_A_TEAPOT
// }