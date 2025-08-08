use axum::{Json, response::IntoResponse, http::StatusCode, extract::State};
use jsonwebtoken::{encode, EncodingKey, Header};
use polodb_core::{CollectionT, bson::doc, Collection};
use crate::AppState;
use serde_json::{json};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use cratis_core::error::{display_msg, CratisError, CratisErrorLevel};

// Request Structs
#[derive(Deserialize)]
pub struct RegisterRequestData {
    hostname: String,
    os: String,
}

// Collection Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    device_id: String,
    auth_token: String,
}

// JWT Struct
#[derive(Debug, Serialize)]
struct Claims {
    device_id: String
}

pub async fn register(State(state): State<AppState>, Json(payload): Json<RegisterRequestData>) -> impl IntoResponse {
    // Validate input
    if payload.hostname.is_empty() || payload.os.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "hostname and os are required"})));
    }

    // Generate device id from hostname and os
    let device_id: String = generate_device_id(payload.hostname, payload.os);

    // Define collection and check if device id already exists in database
    let collection: Collection<Device> = state.db.collection::<Device>("devices");
    let check_id: Result<Option<Device>, polodb_core::Error> = collection.find_one(doc! { "device_id": &device_id });

    if let Ok(Some(_)) = check_id {
        return (StatusCode::CONFLICT, Json(json!({ "error": "Device already exists" })))
    }

    // Handle database error
    if let Err(e) = check_id {
        display_msg(Some(&CratisError::DatabaseError(e.to_string())), CratisErrorLevel::Warning, None);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal Server Error" })))
    }

    // Generate new JWT for device
    let jwt: String = match generate_jwt(device_id.clone()) {
        Some(token) => token,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal Server Error" })))
    };

    // Insert device_id and jwt into db
    let result = collection.insert_one(Device {device_id, auth_token: jwt.clone()});
    if let Err(e) = result {
        display_msg(Some(&CratisError::DatabaseError(format!("Error inserting data: {}", e))), CratisErrorLevel::Warning, None);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal Server Error" })))
    }

    // Return if successful
    (StatusCode::OK, Json(json!({ "status": "ok", "token": jwt })))
}

fn generate_device_id(hostname: String, os: String) -> String {
    let combined: String = format!("{}/{}", hostname, os);
    let hash = Sha256::digest(combined);
    Uuid::new_v5(&Uuid::NAMESPACE_URL, &hash).to_string()
}

fn generate_jwt(device_id: String) -> Option<String> {
    match std::env::var("JWT_SECRET") {
        Ok(secret) => {
            let encoding_key: EncodingKey = EncodingKey::from_secret(secret.as_bytes());

            let claims = Claims { device_id };

            let token = encode(&Header::default(), &claims, &encoding_key)
                .map_err(|e| {display_msg(Some(&CratisError::TokenError(e.to_string())), CratisErrorLevel::Warning, None)});

            Some(token.unwrap())
        }
        Err(e) => {
            display_msg(Some(&CratisError::TokenError(e.to_string())), CratisErrorLevel::Warning, None);
            None
        }
    }
}