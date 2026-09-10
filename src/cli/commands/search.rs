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

/// Discover searchable sites through the methods exposed by the native bridge.
fn get_all_searchable_sites(
    mut request: impl FnMut(&str, serde_json::Value) -> Result<serde_json::Value>,
) -> Result<Vec<String>> {
    let site_list = request("getSiteList", serde_json::Value::Null)?;

    let sites = site_list
        .as_array()
        .context("invalid site list response: expected an array")?;

    let mut searchable = Vec::new();
    for site in sites {
        if site
            .get("offline")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            continue;
        }
        let site_id = site
            .get("id")
            .and_then(|v| v.as_str())
            .context("site list entry is missing a string id")?;
        let config = request("getSiteUserConfig", serde_json::json!({"siteId": site_id}))
            .with_context(|| format!("failed to get search config for site '{site_id}'"))?;
        let allow_search = config
            .get("allowSearch")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_offline = config
            .get("isOffline")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if allow_search && !is_offline {
            searchable.push(site_id.to_string());
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
        let all = get_all_searchable_sites(|method, params| {
            send::send_raw(instance, timeout, method, params)
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::constants::ALLOWED_METHODS;
    use serde_json::json;

    #[test]
    fn discovers_only_searchable_online_sites_using_allowed_methods() {
        let configs = json!({
            "enabled": {"allowSearch": true},
            "disabled": {"allowSearch": false},
            "default": {},
            "offline": {"allowSearch": true, "isOffline": true},
            "dead": {"allowSearch": true},
            "second": {"allowSearch": true, "isOffline": false}
        });
        let sites = get_all_searchable_sites(|method, params| {
            anyhow::ensure!(
                ALLOWED_METHODS.contains(&method),
                "[METHOD_NOT_ALLOWED] method '{method}' is not in the allowlist"
            );
            match method {
                "getSiteList" => {
                    assert!(params.is_null());
                    Ok(configs
                        .as_object()
                        .unwrap()
                        .keys()
                        .map(|id| json!({"id": id, "offline": id == "dead"}))
                        .collect())
                }
                "getSiteUserConfig" => {
                    let id = params["siteId"].as_str().unwrap();
                    assert_ne!(id, "dead", "offline sites should not need config requests");
                    Ok(configs[id].clone())
                }
                _ => panic!("unexpected method: {method}"),
            }
        })
        .unwrap();

        assert_eq!(sites, ["enabled", "second"]);
    }

    #[test]
    fn empty_site_list_needs_no_config_requests() {
        let sites = get_all_searchable_sites(|method, _| {
            assert_eq!(method, "getSiteList");
            Ok(json!([]))
        })
        .unwrap();

        assert!(sites.is_empty());
    }

    #[test]
    fn rejects_malformed_site_lists() {
        for response in [json!(null), json!({}), json!([{}]), json!([{"id": 123}])] {
            let error = get_all_searchable_sites(|method, _| {
                assert_eq!(method, "getSiteList");
                Ok(response.clone())
            })
            .unwrap_err();

            assert!(error.to_string().contains("site list"));
        }
    }

    #[test]
    fn propagates_discovery_request_errors() {
        for failing_method in ["getSiteList", "getSiteUserConfig"] {
            let error = get_all_searchable_sites(|method, _| {
                anyhow::ensure!(method != failing_method, "fixture request failure");
                Ok(json!([{"id": "enabled", "offline": false}]))
            })
            .unwrap_err();

            assert!(format!("{error:#}").contains("fixture request failure"));
            if failing_method == "getSiteUserConfig" {
                assert!(error.to_string().contains("site 'enabled'"));
            }
        }
    }
}
