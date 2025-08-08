use axum::{Router, routing::post};
use crate::handler::authentication::register;
use cratis_core::error::{CratisError, display_msg, CratisErrorLevel};
use polodb_core::Database;
use std::sync::Arc;

mod handler;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Database>
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    match dotenv::from_filename("cratis-api/.env") {
        Ok(path) => println!("Loaded .env from: {:?}", path),
        Err(e) => println!("Failed to load .env: {}", e),
    }

    // Open database
    let mut db_path = std::env::current_dir().unwrap();
    db_path.push("cratis-api");
    db_path.push("database");
    db_path.push("cratis.db");
    let db: Database = Database::open_path(db_path).unwrap();

    // Create new app state
    let app_state: AppState = AppState { db: Arc::new(db) };

    // Router
    let app = Router::new()
        .route("/register", post(register))
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}