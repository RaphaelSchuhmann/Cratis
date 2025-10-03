use clap_derive::{Parser, Subcommand};
use cratis_core::backup::backup;
use cratis_core::config::get_config_cli;
use cratis_core::error::{CratisError, CratisErrorLevel, CratisResult, display_msg};
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use sysinfo::System;

#[derive(Parser)]
#[command(name = "cratis.db")]
# [command(about = "Manage your backups", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Registers device on server
    Register,
    // Immediately trigger a backup based on the current configuration
    BackupNow,
    // Restore a specific snapshot for a given file path
    RestoreSnapshot {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
    },
    // List all available versions/snapshots of a given file path
    ListVersions {
        #[arg(short, long)]
        file: String,
    },
    // Print the currently loaded configuration
    ShowConfig,
    // Send a test request to verify server connectivity and token validity
    PingServer,
}

/// Registers the current device with the Cratis server.
///
/// This function collects system information (hostname and OS) and sends a registration
/// request to the server. Upon successful registration, it returns an authentication token
/// that can be used for subsequent API calls.
///
/// # Returns
///
/// * `Ok(String)` - The authentication token received from the server
/// * `Err(CratisError)` - If registration fails due to:
///   - Network connectivity issues
///   - Server not found (404)
///   - Device already registered (409)
///   - Invalid server response
///   - Unable to retrieve system information
///
/// # Examples
///
/// ```ignore
/// match register().await {
///     Ok(token) => println!("Registration successful! Token: {}", token),
///     Err(e) => eprintln!("Registration failed: {}", e),
/// }
/// ```
///
/// # Network Requirements
///
/// * Server must be running at `http://localhost:8080`
/// * `/register` endpoint must be available
/// * Device must have network connectivity
///
/// # System Information Collected
///
/// * Hostname - Retrieved from system information
/// * Operating System - Retrieved from system information
pub async fn register() -> CratisResult<String> {
    let hostname: String = System::host_name().ok_or(CratisError::Unknown)?;
    let os: String = System::name().ok_or(CratisError::Unknown)?;

    let mut device_info: HashMap<String, String> = HashMap::new();
    (&mut device_info).insert("hostname".to_string(), hostname);
    (&mut device_info).insert("os".to_string(), os);

    let client: Client = Client::new();
    let response: Response = client
        .post(format!("{}/register", get_config_cli().server.address))
        .json(&device_info)
        .send()
        .await
        .map_err(|_| CratisError::RequestError("Unable to send request"))?;

    let status: StatusCode = response.status();
    let response_body: String = response
        .text()
        .await
        .map_err(|_| CratisError::RequestError("Invalid response"))?;

    if status.is_success() {
        let json_value: Value = serde_json::from_str(&response_body)
            .map_err(|_| CratisError::RequestError("Invalid response"))?;

        if let Some(token) = json_value.get("token").and_then(|v| v.as_str()) {
            Ok(token.to_string())
        } else {
            Err(CratisError::RequestError(
                "Invalid response: Token missing!",
            ))
        }
    } else if status == StatusCode::NOT_FOUND {
        Err(CratisError::RequestError("Server not found"))
    } else if status == StatusCode::CONFLICT {
        Err(CratisError::RequestError("Device already registered"))
    } else {
        Err(CratisError::RequestError("Invalid response"))
    }
}

pub async fn ping_server() -> CratisResult<String> {
    let client: Client = Client::new();
    let response: Response = client
        .get(format!("{}/ping", get_config_cli().server.address))
        .send()
        .await
        .map_err(|_| {
            CratisError::ConnectionIssue("Unable to send request, server is not reachable!")
        })?;

    let status: StatusCode = response.status();

    if status == StatusCode::OK {
        Ok("Server is reachable!".to_string())
    } else {
        Err(CratisError::Unknown)
    }
}

pub async fn backup_now() -> CratisResult<String> {
    let status: http::status::StatusCode = backup().await;

    match status {
        s if s.is_success() => Ok("Files backed up successfully!".to_string()),
        http::status::StatusCode::NOT_FOUND => Err(CratisError::RequestError("Server not found")),
        http::status::StatusCode::UNAUTHORIZED => Err(CratisError::RequestError("Unauthorized")),
        _ => Err(CratisError::RequestError("Invalid response")),
    }
}
