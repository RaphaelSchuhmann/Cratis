use clap::{Parser};
use cratis_core::error::{display_msg, CratisErrorLevel, CratisResult};
use cratis_core::config::{update_config, load_config, TEMP_CONFIG_PATH, get_config};
use crate::cli::{Commands, register, backup_now};
use serde_yaml::Value;

mod cli;

#[tokio::main]
async fn main() {
    load_config(TEMP_CONFIG_PATH);
    let cli_ = cli::Cli::parse();

    match cli_.command {
        Commands::Register {} => {
            display_msg(None, CratisErrorLevel::Info, Some("Registering...".to_string()));

            let result: CratisResult<String> = register().await;
            match result {
                Ok(_) => {
                    display_msg(None, CratisErrorLevel::Info, Some("Registered successfully!".to_string()));
                    let result: CratisResult<()> = update_config("server.auth_token", Value::String(result.unwrap()));
                    match result {
                        Ok(_) => {
                            display_msg(None, CratisErrorLevel::Info, Some("Updated config successfully!".to_string()));
                        },
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
            println!("Ping server");
        }
        Commands::ShowConfig {} => {
            println!("Getting Config");
        }
    }
}