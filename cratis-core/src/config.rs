#![allow(dead_code)]
use serde::Deserialize;
use once_cell::sync::OnceCell;
use serde_yaml::{Value};
use std::fs;
use crate::error::{display_msg, CratisError, CratisErrorLevel, CratisResult};

// TODO: Remove this later on when a proper .yml selection is implemented
pub static TEMP_CONFIG_PATH: &str = "/home/raphael/Development/Cratis/cratis.yml";
pub static TEMP_API_CONFIG_PATH: &str = "/home/raphael/Development/Cratis/cratis-api/config.yml";

#[derive(Debug, Deserialize)]
pub struct CratisConfig {
    pub client: ClientConfig,
    pub backup: BackupConfig,
    pub server: ServerConfig,
    pub advanced: Option<AdvancedConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BackupConfig {
    pub mode: BackupMode,
    pub watch_directories: Vec<String>,
    pub exclude: Option<Vec<String>>,
    pub interval_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupMode {
    Full,
    Incremental,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub auth_token: String
}

#[derive(Debug, Deserialize)]
pub struct AdvancedConfig {
    pub max_file_size_mb: Option<u64>,
    pub retry_attempts: Option<u32>,
    pub retry_delay_seconds: Option<u64>,
    pub enable_notifications: Option<bool>
}

#[derive(Debug, Deserialize)]
pub struct CratisServerConfig {
    pub port: u16,
    pub env: String,
    pub db: String,
}

static CONFIG_CLI: OnceCell<CratisConfig> = OnceCell::new();
static CONFIG_API: OnceCell<CratisServerConfig> = OnceCell::new();


/// Loads configuration from a YAML file into global static storage.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
/// * `api` - If true, loads server config; if false, loads client config
///
/// # Panics
///
/// Panics if the file cannot be read or parsed.
pub fn load_config(path: &str, api: bool) {
    let contents = fs::read_to_string(path).expect("Failed to read config file");

    if api {
        let parsed: CratisServerConfig = serde_yaml::from_str(&contents).expect("Invalid config format");
        CONFIG_API.set(parsed).expect("Config initialized");
        return;
    }

    let parsed: CratisConfig = serde_yaml::from_str(&contents).expect("Invalid config format");
    CONFIG_CLI.set(parsed).expect("Config initialized");
}


pub fn get_config_cli() -> &'static CratisConfig {
    if CONFIG_CLI.get().is_none() {
        load_config(TEMP_CONFIG_PATH, false);

        if CONFIG_CLI.get().is_none() {
            display_msg(Some(&CratisError::ConfigError("Unable to load config".to_string())), CratisErrorLevel::Fatal, None);
            unreachable!()
        } else {
            CONFIG_CLI.get().unwrap()
        }
    } else {
        CONFIG_CLI.get().unwrap()
    }
}

pub fn get_config_api() -> &'static CratisServerConfig {
    if CONFIG_API.get().is_none() {
        load_config(TEMP_API_CONFIG_PATH, true);
        
        if CONFIG_API.get().is_none() {
            display_msg(Some(&CratisError::ConfigError("Unable to load config".to_string())), CratisErrorLevel::Fatal, None);
            unreachable!()
        } else {
            CONFIG_API.get().unwrap()
        }
    } else {
        CONFIG_API.get().unwrap()
    }
}

/// Updates a configuration value in the YAML file using a dot-separated key path.
///
/// This function reads the existing configuration file, navigates to the specified
/// key using dot notation, updates the value, and writes the changes back to the file.
/// Creates nested keys if they don't exist.
///
/// # Arguments
///
/// * `key_path` - Dot-separated path to the configuration key (e.g., "server.address")
/// * `new_value` - The new value to set, as a serde_yaml::Value
///
/// # Returns
///
/// * `Ok(())` - If the configuration was successfully updated
/// * `Err(CratisError)` - If file operations fail or the key path is invalid
///
/// # Examples
///
/// ```ignore
/// use serde_yaml::Value;
///
/// // Update server address
/// let new_addr = Value::String("0.0.0.0:9000".to_string());
/// update_config("server.address", new_addr)?;
///
/// // Update nested configuration
/// let new_mode = Value::String("incremental".to_string());
/// update_config("backup.mode", new_mode)?;
/// ```
///
/// # Errors
///
/// Returns `CratisError` if:
/// * The configuration file cannot be read or written
/// * The YAML parsing fails
/// * The key path points to an invalid location in the configuration structure
pub fn update_config(key_path: &str, config_path: &str, new_value: Value) -> CratisResult<()> {
    // Read existing YAML file
    let file_content: String = fs::read_to_string(config_path)?;
    let mut yaml_value: Value = serde_yaml::from_str(&file_content)?;

    // Split the key path by '.' for nested access
    let keys: Vec<&str> = key_path.split('.').collect();

    // Traverse the Value tree and update the field
    fn update_recursive(value: &mut Value, keys: &[&str], new_value: Value) -> CratisResult<()> {
        if keys.is_empty() {
            *value = new_value;
            return Ok(());
        }
        match value {
            Value::Mapping(map) => {
                let key = Value::String(keys[0].to_string());
                if let Some(v) = map.get_mut(&key) {
                    update_recursive(v, &keys[1..], new_value)
                } else {
                    // If key doesn't exist, create it
                    map.insert(key.clone(), Value::Null);
                    update_recursive(map.get_mut(&key).unwrap(), &keys[1..], new_value)
                }
            },
            _ => Err(CratisError::ConfigError("Error while updating config!".to_string())),
        }
    }

    update_recursive(&mut yaml_value, &keys, new_value)?;

    // Write back to file
    let new_yaml_str: String = serde_yaml::to_string(&yaml_value)?;
    fs::write(config_path, new_yaml_str)?;

    Ok(())
}