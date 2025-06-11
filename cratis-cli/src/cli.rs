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
    BackupNow,
    RestoreSnapshot {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
    },
    ListVersions {
        #[arg(short, long)]
        file: String,
    },
}