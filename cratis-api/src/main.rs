use axum::{Router, routing::post, middleware};
use crate::handler::authentication::{authenticate_middleware, register};
use polodb_core::Database;
use std::sync::Arc;
use axum::response::IntoResponse;
use once_cell::sync::Lazy;
use cratis_core::error::{display_msg, CratisError, CratisErrorLevel};

mod handler;

// Database:
pub static DB: Lazy<Arc<Database>> = Lazy::new(|| {
    let mut db_path = std::env::current_dir().unwrap();
    db_path.push("cratis-api");
    db_path.push("database");
    db_path.push("cratis.db");
    Arc::new(Database::open_path(db_path).expect("Failed to open DB"))
});

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    match dotenv::from_filename("cratis-api/.env") {
        Ok(path) => println!("Loaded .env from: {:?}", path),
        Err(e) => display_msg(Some(&CratisError::EnvError(e.to_string())), CratisErrorLevel::Fatal, None),
    }

    // Router
    // let auth_routes = Router::new()
    //     // Put any routes that need authentication here
    //     .route_layer(middleware::from_fn(authenticate_middleware));

    let public_routes = Router::new()
        .route("/register", post(register));

    let app = Router::new()
        .merge(public_routes);
        // .merge(auth_routes);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}