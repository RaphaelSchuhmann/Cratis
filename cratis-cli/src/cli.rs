use clap_derive::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cratis")]
# [command(about = "Manage your backups", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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