use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct KeepUploadArgs {
    #[command(subcommand)]
    pub command: KeepUploadCommand,
}

#[derive(Subcommand)]
pub enum KeepUploadCommand {
    /// List all keep-upload tasks
    List,
    /// Get a specific task
    Get { task_id: String },
    /// Create a new task from a JSON file
    Create {
        #[arg(long = "file")]
        file: PathBuf,
    },
    /// Update a task from a JSON file
    Update {
        #[arg(long = "file")]
        file: PathBuf,
    },
    /// Delete a task
    Delete { task_id: String },
    /// Clear all tasks
    Clear,
}

pub fn run(args: KeepUploadArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    match args.command {
        KeepUploadCommand::List => {
            send::send_and_print(instance, timeout, format, "getKeepUploadTasks", serde_json::Value::Null)?;
        }
        KeepUploadCommand::Get { task_id } => {
            send::send_and_print(instance, timeout, format, "getKeepUploadTaskById", serde_json::json!(task_id))?;
        }
        KeepUploadCommand::Create { file } => {
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let task: serde_json::Value =
                serde_json::from_str(&content).context("failed to parse task JSON")?;
            send::send_and_print(instance, timeout, format, "createKeepUploadTask", task)?;
        }
        KeepUploadCommand::Update { file } => {
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let task: serde_json::Value =
                serde_json::from_str(&content).context("failed to parse task JSON")?;
            send::send_and_print(instance, timeout, format, "updateKeepUploadTask", task)?;
        }
        KeepUploadCommand::Delete { task_id } => {
            send::send_and_print(instance, timeout, format, "deleteKeepUploadTask", serde_json::json!(task_id))?;
        }
        KeepUploadCommand::Clear => {
            send::send_and_print(instance, timeout, format, "clearKeepUploadTasks", serde_json::Value::Null)?;
        }
    }
    Ok(())
}
