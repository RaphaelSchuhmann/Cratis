use clap::{Parser};
use cratis_core::error::{display_msg, CratisErrorLevel, CratisResult};
use cratis_core::config::{update_config, load_config, TEMP_CONFIG_PATH};
use crate::cli::{Commands, register, backup_now, ping_server};
use serde_yaml::Value;

mod cli;

#[tokio::main]
async fn main() {
    load_config(TEMP_CONFIG_PATH);
    let cli_ = cli::Cli::parse();

    match cli_.command {
        Commands::Register {} => {
            display_msg(None, CratisErrorLevel::Info, Some("Registering...".to_string()));

            match register().await {
                Ok(token) => {
                    display_msg(None, CratisErrorLevel::Info, Some("Registered successfully!".to_string()));
                    match update_config("server.auth_token", Value::String(token)) {
                        Ok(_) => display_msg(None, CratisErrorLevel::Info, Some("Updated config successfully!".to_string())),
                        Err(e) => display_msg(Some(&e), CratisErrorLevel::Warning, None),
                    }
                }
                Err(e) => display_msg(Some(&e), CratisErrorLevel::Warning, None),
            }
        }
        Commands::BackupNow {} => {
            display_msg(None, CratisErrorLevel::Info, Some("Starting backup".to_string()));

            let result: CratisResult<String> = backup_now().await;
            match result {
                Ok(_) => display_msg(None, CratisErrorLevel::Info, Some(result.unwrap())),
                Err(e) => display_msg(Some(&e), CratisErrorLevel::Warning, None),
            }
        }
        Commands::RestoreSnapshot { from, to } => {
            println!("Restore snapshot from {} to {}", from, to);
        }
        Commands::ListVersions { file} => {
            println!("List versions of {}", file);
        }
        Commands::PingServer {} => {
            display_msg(None, CratisErrorLevel::Info, Some("Pinging server...".to_string()));

            match ping_server().await {
                Ok(msg) => display_msg(None, CratisErrorLevel::Info, Some(msg)),
                Err(e) => display_msg(Some(&e), CratisErrorLevel::Fatal, None)
            }
        }
        Commands::ShowConfig {} => {
            println!("Getting Config");
        }
    }
}