use cratis_core::error::{display_msg, CratisError, CratisErrorLevel, CratisResult};
use cratis_core::utils::{is_path_file, get_files_in_directory, load_file};
use clap_derive::{Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
use tokio::fs::File as TokioFile;
use tokio_util::io::ReaderStream;
use reqwest::{Client, Response, StatusCode};
use sysinfo::System;
use std::collections::HashMap;
use serde_json::Value;

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
    let response: Response = client.post("http://localhost:8080/register")
        .json(&device_info)
        .send()
        .await
        .map_err(|_| CratisError::RequestError("Unable to send request"))?;

    let status: StatusCode = response.status();
    let response_body: String = response.text().await.map_err(|_| CratisError::RequestError("Invalid response"))?;

    if status.is_success() {
        let json_value: Value = serde_json::from_str(&response_body).map_err(|_| CratisError::RequestError("Invalid response"))?;

        if let Some(token) = json_value.get("token").and_then(|v| v.as_str()) {
            return Ok(token.to_string());
        } else {
            Err(CratisError::RequestError("Invalid response: Token missing!"))
        }

    } else if status == StatusCode::NOT_FOUND {
        Err(CratisError::RequestError("Server not found"))
    } else if status == StatusCode::CONFLICT {
        Err(CratisError::RequestError("Device already registered"))
    } else {
        Err(CratisError::RequestError("Invalid response"))
    }
}

pub async fn backup_now() -> CratisResult<String> {
    let config = cratis_core::config::get_config();
    let watch_dirs = &config.backup.watch_directories;

    let mut files_to_load: Vec<PathBuf> = Vec::new();

    for dir in watch_dirs {
        if is_path_file(dir) {
            files_to_load.push(PathBuf::from(dir));
        } else {
            let files: CratisResult<Vec<PathBuf>> = get_files_in_directory(dir);
            match files {
                Ok(files) => {
                    files_to_load.extend(files);
                }
                Err(e) => {
                    display_msg(Some(&e), CratisErrorLevel::Warning, None)
                }
            }
        }
    }

    let mut loaded_files: Vec<(File, String)> = Vec::new();

    for file in files_to_load {
        let loaded_file = load_file(file);
        match loaded_file {
            Ok(file) => {
                loaded_files.push(file);
            }
            Err(e) => {
                display_msg(Some(&e), CratisErrorLevel::Warning, None)
            }
        }
    }

    // Put loaded files into request body
    let client = reqwest::Client::new();
    let api_url = "http://localhost:8080/upload"; // TODO: Make this configurable
    let auth_token = "test"; // TODO: Load it from config, write auth token generator

    let mut form = reqwest::multipart::Form::new();

    for (std_file, file_name) in loaded_files {
        let tokio_file: TokioFile = TokioFile::from_std(std_file);
        let file_body_stream = ReaderStream::new(tokio_file);
        let body = reqwest::Body::wrap_stream(file_body_stream);
        let file_part = reqwest::multipart::Part::stream(body).file_name(file_name).mime_str("application/octet-stream").map_err(|_| CratisError::RequestError("Unable to send file"))?;

        form = form.part("files", file_part);
    }

    // Send request
    let response = client.post(api_url)
        .bearer_auth(auth_token)
        .multipart(form)
        .send()
        .await
        .map_err(|_| CratisError::RequestError("Unable to send request"))?;

    let status = response.status();
    let response_body: String = response.text().await.map_err(|_| CratisError::RequestError("Invalid response"))?;

    if status.is_success() {
        Ok(response_body)
    } else if status == reqwest::StatusCode::NOT_FOUND {
        Err(CratisError::RequestError("Server not found"))
    } else if status == reqwest::StatusCode::UNAUTHORIZED {
        Err(CratisError::RequestError("Unauthorized"))
    } else {
        Err(CratisError::RequestError("Invalid response"))
    }
}