use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::cli::last_search;
use crate::cli::output::OutputFormat;
use crate::cli::send;

#[derive(Args)]
pub struct SearchArgs {
    /// Search keyword
    keyword: String,

    /// Site ID(s) to search. Specify multiple times for multi-site search.
    #[arg(long = "site", required = true)]
    sites: Vec<String>,

    /// Path to a JSON file containing a full IAdvancedSearchRequestConfig
    #[arg(long = "entry-file")]
    entry_file: Option<PathBuf>,
}

pub fn run(args: SearchArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    let search_entry: serde_json::Value = if let Some(path) = &args.entry_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read entry file: {}", path.display()))?;
        serde_json::from_str(&content).context("failed to parse entry file JSON")?
    } else {
        serde_json::json!({})
    };

    let instance_id = send::resolve_instance_id(instance)?;
    let mut all_results: Vec<serde_json::Value> = Vec::new();

    for site_id in &args.sites {
        let params = serde_json::json!({
            "siteId": site_id,
            "keyword": args.keyword,
            "searchEntry": search_entry,
        });

        let result = send::send_raw(instance, timeout, "getSiteSearchResult", params)?;

        // Extract the data array from ISearchResult
        if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
            // Tag each result with the site it came from (for display)
            for item in data {
                let mut item = item.clone();
                if let serde_json::Value::Object(ref mut obj) = item {
                    obj.entry("_siteId").or_insert(serde_json::json!(site_id));
                }
                all_results.push(item);
            }
        }

        // Print status message for this site
        let status = result
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        let count = result
            .get("data")
            .and_then(|d| d.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        eprintln!("[{site_id}] {status}: {count} results");
    }

    // Cache results for `ptd download <index>`
    let cache_value = serde_json::to_value(&all_results)?;
    if let Err(e) = last_search::save(&instance_id, &cache_value) {
        eprintln!("warning: failed to cache search results: {e}");
    }

    // Print combined results
    let output = serde_json::to_value(&all_results)?;
    crate::cli::output::print_value(&output, format)?;

    Ok(())
}
