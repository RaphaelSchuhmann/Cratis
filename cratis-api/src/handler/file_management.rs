use axum::{extract::Multipart, response::IntoResponse, http::StatusCode};
use polodb_core::{CollectionT, bson::doc, Collection};
use serde::{Deserialize, Serialize};
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;

// Collection Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    device_id: String,
}

pub async fn backup(mut multipart: Multipart) -> impl IntoResponse {
    // file_name, file_path, file_size
    let mut metadata: Vec<(String, String, u32)> = Vec::new();
    
}