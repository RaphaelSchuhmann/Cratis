use clap::{Parser};
use cratis_core::error::{display_msg, CratisErrorLevel, CratisResult};
use crate::cli::{Commands, backup_now};

mod cli;

#[tokio::main]
async fn main() {
    let cli_ = cli::Cli::parse();

    match cli_.command {
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