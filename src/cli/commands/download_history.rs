use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct DownloadHistoryArgs {
    #[command(subcommand)]
    pub command: Option<DownloadHistoryCommand>,
}

#[derive(Subcommand)]
pub enum DownloadHistoryCommand {
    /// Get a specific download history entry
    Get { download_id: String },
    /// Delete a specific download history entry
    Delete { download_id: String },
    /// Clear all download history
    Clear,
}

pub fn run(args: DownloadHistoryArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    match args.command {
        None => {
            // List all download history
            send::send_and_print(instance, timeout, format, "getDownloadHistory", serde_json::Value::Null)?;
        }
        Some(DownloadHistoryCommand::Get { download_id }) => {
            send::send_and_print(instance, timeout, format, "getDownloadHistoryById", serde_json::json!(download_id))?;
        }
        Some(DownloadHistoryCommand::Delete { download_id }) => {
            send::send_and_print(instance, timeout, format, "deleteDownloadHistoryById", serde_json::json!(download_id))?;
        }
        Some(DownloadHistoryCommand::Clear) => {
            send::send_and_print(instance, timeout, format, "clearDownloadHistory", serde_json::Value::Null)?;
        }
    }
    Ok(())
}
