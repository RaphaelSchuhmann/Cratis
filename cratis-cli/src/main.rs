use clap::{Parser};
use crate::cli::Commands;

mod cli;

#[tokio::main]
async fn main() {
    let cli_ = cli::Cli::parse();

    match cli_.command {
        Commands::BackupNow {} => {
            println!("Backup now");
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