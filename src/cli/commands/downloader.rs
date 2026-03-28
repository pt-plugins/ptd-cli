use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct DownloaderArgs {
    #[command(subcommand)]
    pub command: DownloaderCommand,
}

#[derive(Subcommand)]
pub enum DownloaderCommand {
    /// Get downloader status
    Status { downloader_id: String },
    /// Get downloader config
    Config { downloader_id: String },
    /// Get downloader version
    Version { downloader_id: String },
}

pub fn run(args: DownloaderArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    match args.command {
        DownloaderCommand::Status { downloader_id } => {
            send::send_and_print(instance, timeout, format, "getDownloaderStatus", serde_json::json!(downloader_id))?;
        }
        DownloaderCommand::Config { downloader_id } => {
            send::send_and_print(instance, timeout, format, "getDownloaderConfig", serde_json::json!(downloader_id))?;
        }
        DownloaderCommand::Version { downloader_id } => {
            send::send_and_print(instance, timeout, format, "getDownloaderVersion", serde_json::json!(downloader_id))?;
        }
    }
    Ok(())
}
