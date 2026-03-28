use anyhow::Result;
use clap::{Args, Subcommand};

use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct SiteArgs {
    #[command(subcommand)]
    pub command: SiteCommand,
}

#[derive(Subcommand)]
pub enum SiteCommand {
    /// Get site user config
    Config { site_id: String },
    /// Get site favicon (base64)
    Favicon {
        site_id: String,
        /// Force refresh the cached favicon
        #[arg(long)]
        flush: bool,
    },
}

pub fn run(args: SiteArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    match args.command {
        SiteCommand::Config { site_id } => {
            send::send_and_print(instance, timeout, format, "getSiteUserConfig", serde_json::json!({"siteId": site_id}))?;
        }
        SiteCommand::Favicon { site_id, flush } => {
            send::send_and_print(instance, timeout, format, "getSiteFavicon", serde_json::json!({"site": site_id, "flush": flush}))?;
        }
    }
    Ok(())
}
