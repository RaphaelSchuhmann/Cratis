#[allow(dead_code)]
use axum::{Json, response::IntoResponse, http::StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http::Request;
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation, Algorithm};
use polodb_core::{CollectionT, bson::doc, Collection};
use serde_json::{json};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use cratis_core::config::get_config_api;
use cratis_core::error::{display_msg, CratisError, CratisErrorLevel};
use crate::DB;

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
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    device_id: String
}

/// Handles device registration requests.
///
/// This endpoint registers a new device by generating a unique device ID from the hostname
/// and OS, checking for duplicates in the database, and creating a JWT token for authentication.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `payload` - JSON payload containing hostname and OS information
///
/// # Returns
///k
/// * `200 OK` with JWT token if registration is successful
/// * `400 Bad Request` if hostname or OS is empty
/// * `409 Conflict` if device already exists
/// * `500 Internal Server Error` for database or JWT generation errors
///
/// # Examples
///
/// ```json
/// // Request
/// {
///   "hostname": "my-laptop",
///   "os": "linux"
/// }
///
/// // Response
/// {
///   "status": "ok",
///   "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
/// }
/// ```
pub async fn register(Json(payload): Json<RegisterRequestData>) -> impl IntoResponse {
    // Validate input
    if payload.hostname.is_empty() || payload.os.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "hostname and os are required"})));
    }

    // Generate device id from hostname and os
    let device_id: String = generate_device_id(payload.hostname, payload.os);

    // Define collection and check if device id already exists in database
    let collection: Collection<Device> = DB.collection::<Device>("devices");
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

pub async fn authenticate_middleware(mut req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get(http::header::AUTHORIZATION).and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            match decode_token(token) {
                Ok(claims) => {
                    // Check if device_id is in db
                    let collection: Collection<Device> = DB.collection::<Device>("devices");
                    let result: Result<Option<Device>, polodb_core::Error> = collection.find_one(doc! { "device_id": &claims.device_id });

                    // Handle database error
                    if let Err(e) = result {
                        display_msg(Some(&CratisError::DatabaseError(e.to_string())), CratisErrorLevel::Warning, None);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }

                    if let Ok(Some(_)) = result {
                        req.extensions_mut().insert(claims);
                        return Ok(next.run(req).await);
                    } else {
                        return Err(StatusCode::UNAUTHORIZED)
                    }
                }
                Err(_) => return Err(StatusCode::UNAUTHORIZED)
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Generates a unique device ID from hostname and OS information.
///
/// Creates a deterministic UUID v5 by combining the hostname and OS into a string,
/// hashing it with SHA-256, and generating a UUID using the URL namespace.
///
/// # Arguments
///
/// * `hostname` - The device hostname
/// * `os` - The operating system name
///
/// # Returns
///
/// A string representation of the generated UUID
///
/// # Examples
///
/// ```ignore
/// let device_id = generate_device_id("laptop".to_string(), "linux".to_string());
/// assert_eq!(device_id.len(), 36); // UUID string length
/// ```
fn generate_device_id(hostname: String, os: String) -> String {
    let combined: String = format!("{}/{}", hostname, os);
    let hash = Sha256::digest(combined);
    Uuid::new_v5(&Uuid::NAMESPACE_URL, &hash).to_string()
}

/// Generates a JWT token for device authentication.
///
/// Creates a JSON Web Token containing the device ID as a claim, signed with
/// the JWT_SECRET environment variable. Returns None if the secret is not set
/// or token generation fails.
///
/// # Arguments
///
/// * `device_id` - The unique device identifier to include in the token
///
/// # Returns
///
/// * `Some(String)` - The generated JWT token
/// * `None` - If JWT_SECRET is not set or token generation fails
///
/// # Environment Variables
///
/// * `JWT_SECRET` - Secret key used for signing the JWT token
///
/// # Examples
///
/// ```ignore
/// std::env::set_var("JWT_SECRET", "my-secret-key");
/// let token = generate_jwt("device-123".to_string());
/// assert!(token.is_some());
/// ```
fn generate_jwt(device_id: String) -> Option<String> {
    let secret: String = get_config_api().settings.jwt.clone();

    if secret.is_empty() {
        display_msg(Some(&CratisError::TokenError("JWT Secret is empty!".to_string())), CratisErrorLevel::Warning, None);
        return None
    }

    let encoding_key: EncodingKey = EncodingKey::from_secret(secret.as_bytes());
    let claims = Claims { device_id };
    match encode(&Header::default(), &claims, &encoding_key) {
        Ok(t) => Some(t),
        Err(e) => {
            display_msg(Some(&CratisError::TokenError(e.to_string())), CratisErrorLevel::Warning, None);
            None
        }
    }
}

fn decode_token(token: &str) ->  Result<Claims, jsonwebtoken::errors::Error> {
    let secret: String = get_config_api().settings.jwt.clone();

    if secret.is_empty() {
        display_msg(Some(&CratisError::TokenError("JWT Secret is empty!".to_string())), CratisErrorLevel::Warning, None);
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat));
    }

    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation.algorithms = vec![Algorithm::HS256];
    validation.required_spec_claims.remove("exp");

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}