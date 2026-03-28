use anyhow::{Context, Result};

use crate::shared::paths;

/// Save search results to the per-instance cache.
pub fn save(instance_id: &str, results: &serde_json::Value) -> Result<()> {
    let path = paths::last_search_path(instance_id);
    std::fs::create_dir_all(path.parent().unwrap()).context("failed to create cache directory")?;
    let json = serde_json::to_string_pretty(results).context("failed to serialize search results")?;
    std::fs::write(&path, json).context("failed to write last search cache")?;
    Ok(())
}

/// Load the last search results from the per-instance cache.
pub fn load(instance_id: &str) -> Result<Vec<serde_json::Value>> {
    let path = paths::last_search_path(instance_id);
    if !path.exists() {
        anyhow::bail!("no cached search results for this instance. Run 'ptd search' first.");
    }
    let content = std::fs::read_to_string(&path).context("failed to read last search cache")?;
    let results: Vec<serde_json::Value> =
        serde_json::from_str(&content).context("failed to parse last search cache")?;
    Ok(results)
}

/// Get a single torrent entry by index from the last search results.
pub fn get_by_index(instance_id: &str, index: usize) -> Result<serde_json::Value> {
    let results = load(instance_id)?;
    results
        .into_iter()
        .nth(index)
        .with_context(|| format!("index {index} out of range"))
}
