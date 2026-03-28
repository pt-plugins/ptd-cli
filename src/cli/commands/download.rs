use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::cli::last_search;
use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct DownloadArgs {
    /// Index from the last search results
    #[arg(group = "source")]
    index: Option<usize>,

    /// Path to a JSON file containing a full IDownloadTorrentOption
    #[arg(long = "option-file", group = "source")]
    option_file: Option<PathBuf>,

    /// Downloader ID to use (when downloading by index)
    #[arg(long)]
    downloader: Option<String>,

    /// Download the .torrent file locally instead of sending to a downloader
    #[arg(long)]
    local: bool,
}

pub fn run(args: DownloadArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    let params: serde_json::Value = if let Some(path) = &args.option_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read option file: {}", path.display()))?;
        serde_json::from_str(&content).context("failed to parse option file JSON")?
    } else if let Some(index) = args.index {
        let instance_id = send::resolve_instance_id(instance)?;
        let torrent = last_search::get_by_index(&instance_id, index)?;

        let mut option = serde_json::json!({
            "torrent": torrent,
        });

        if let Some(downloader_id) = &args.downloader {
            option["downloaderId"] = serde_json::json!(downloader_id);
        }
        if args.local {
            option["localDownload"] = serde_json::json!(true);
        }

        option
    } else {
        anyhow::bail!("provide either an index from the last search or --option-file");
    };

    send::send_and_print(instance, timeout, format, "downloadTorrent", params)?;
    Ok(())
}
