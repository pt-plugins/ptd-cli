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

    /// Site ID(s) to search. Omit to search all enabled sites.
    #[arg(long = "site")]
    sites: Vec<String>,

    /// Path to a JSON file containing a full IAdvancedSearchRequestConfig
    #[arg(long = "entry-file")]
    entry_file: Option<PathBuf>,

    /// Maximum number of results per site (default: no limit)
    #[arg(long)]
    limit: Option<usize>,
}

/// Fetch all site IDs that have allowSearch enabled from the extension's metadata store.
fn get_all_searchable_sites(instance: Option<&str>, timeout: u64) -> Result<Vec<String>> {
    let metadata = send::send_raw(instance, timeout, "getExtStorage", serde_json::json!("metadata"))?;

    let sites = metadata
        .get("sites")
        .and_then(|s| s.as_object())
        .context("no sites found in metadata")?;

    let mut searchable = Vec::new();
    for (site_id, config) in sites {
        let allow_search = config
            .get("allowSearch")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_offline = config
            .get("isOffline")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if allow_search && !is_offline {
            searchable.push(site_id.clone());
        }
    }
    Ok(searchable)
}

pub fn run(args: SearchArgs, instance: Option<&str>, timeout: u64, format: OutputFormat) -> Result<()> {
    let search_entry: serde_json::Value = if let Some(path) = &args.entry_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read entry file: {}", path.display()))?;
        serde_json::from_str(&content).context("failed to parse entry file JSON")?
    } else {
        serde_json::json!({})
    };

    let sites = if args.sites.is_empty() {
        let all = get_all_searchable_sites(instance, timeout)?;
        eprintln!("Searching {} sites...", all.len());
        all
    } else {
        args.sites.clone()
    };

    if sites.is_empty() {
        anyhow::bail!("no searchable sites found. Open the PT-Depiler extension options page and add sites first, or use --site <siteId> to search a specific site.");
    }

    let instance_id = send::resolve_instance_id(instance)?;
    let mut all_results: Vec<serde_json::Value> = Vec::new();

    for site_id in &sites {
        let params = serde_json::json!({
            "siteId": site_id,
            "keyword": args.keyword,
            "searchEntry": search_entry,
        });

        match send::send_raw(instance, timeout, "getSiteSearchResult", params) {
            Ok(result) => {
                let status = result
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");

                // Extract the data array from ISearchResult
                let total;
                let shown;
                if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
                    total = data.len();
                    let items: Vec<_> = match args.limit {
                        Some(limit) => data.iter().take(limit).collect(),
                        None => data.iter().collect(),
                    };
                    shown = items.len();
                    for item in items {
                        let mut item = item.clone();
                        if let serde_json::Value::Object(ref mut obj) = item {
                            obj.entry("_siteId").or_insert(serde_json::json!(site_id));
                        }
                        all_results.push(item);
                    }
                } else {
                    total = 0;
                    shown = 0;
                }

                if shown < total {
                    eprintln!("[{site_id}] {status}: {shown}/{total} results (limited)");
                } else {
                    eprintln!("[{site_id}] {status}: {total} results");
                }
            }
            Err(e) => {
                eprintln!("[{site_id}] error: {e:#}");
            }
        }
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
