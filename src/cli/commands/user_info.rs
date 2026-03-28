use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct UserInfoArgs {
    #[command(subcommand)]
    pub command: UserInfoCommand,
}

#[derive(Subcommand)]
pub enum UserInfoCommand {
    /// Fetch current user info from site
    Current { site_id: String },
    /// Get stored user info history
    History { site_id: String },
    /// Remove stored user info entries by date
    Remove {
        site_id: String,
        /// Dates to remove
        dates: Vec<String>,
    },
    /// Cancel pending user info queue
    Cancel,
}

pub fn run(args: UserInfoArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    match args.command {
        UserInfoCommand::Current { site_id } => {
            send::send_and_print(instance, timeout, format, "getSiteUserInfoResult", serde_json::json!(site_id))?;
        }
        UserInfoCommand::History { site_id } => {
            send::send_and_print(instance, timeout, format, "getSiteUserInfo", serde_json::json!(site_id))?;
        }
        UserInfoCommand::Remove { site_id, dates } => {
            send::send_and_print(instance, timeout, format, "removeSiteUserInfo", serde_json::json!({"siteId": site_id, "date": dates}))?;
        }
        UserInfoCommand::Cancel => {
            send::send_and_print(instance, timeout, format, "cancelUserInfoQueue", serde_json::Value::Null)?;
        }
    }
    Ok(())
}
