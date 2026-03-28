use std::net::Shutdown;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::shared::paths;
use crate::shared::protocol::{HelloMessage, InstanceRegistry};

/// Attempt to connect to an existing socket to check if it's alive.
fn is_socket_alive(path: &Path) -> bool {
    match StdUnixStream::connect(path) {
        Ok(stream) => {
            let _ = stream.shutdown(Shutdown::Both);
            true
        }
        Err(_) => false,
    }
}

/// Publish instance registry file and prepare the socket path.
/// Returns an error if a live daemon already owns this instance.
pub fn publish(hello: &HelloMessage) -> Result<()> {
    let socket_path = paths::instance_socket_path(&hello.instance_id);
    let registry_path = paths::instance_registry_path(&hello.instance_id);

    // Ensure directories exist
    std::fs::create_dir_all(paths::instances_dir())
        .context("failed to create instances directory")?;
    std::fs::create_dir_all(paths::logs_dir())
        .context("failed to create logs directory")?;

    // Check for existing live daemon
    if socket_path.exists() && is_socket_alive(&socket_path) {
        anyhow::bail!(
            "another daemon is already running for instance {}",
            hello.instance_id
        );
    }

    // Remove stale artifacts
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).ok();
    }
    if registry_path.exists() {
        std::fs::remove_file(&registry_path).ok();
    }

    let now = Utc::now().to_rfc3339();
    let registry = InstanceRegistry {
        instance_id: hello.instance_id.clone(),
        browser: hello.browser.clone(),
        extension_id: hello.extension_id.clone(),
        version: hello.version.clone(),
        socket_path: socket_path.to_string_lossy().into_owned(),
        connected_at: now.clone(),
        last_seen_at: now,
    };

    let json = serde_json::to_string_pretty(&registry)
        .context("failed to serialize registry")?;
    std::fs::write(&registry_path, json)
        .context("failed to write registry file")?;

    Ok(())
}

/// Remove instance socket and registry files.
pub fn cleanup(instance_id: &str) {
    let socket_path = paths::instance_socket_path(instance_id);
    let registry_path = paths::instance_registry_path(instance_id);

    if socket_path.exists() {
        std::fs::remove_file(&socket_path).ok();
    }
    if registry_path.exists() {
        std::fs::remove_file(&registry_path).ok();
    }
}

/// List all registry entries from disk, regardless of health.
pub fn list_all() -> Result<Vec<InstanceRegistry>> {
    let dir = paths::instances_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&dir).context("failed to read instances directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(registry) = serde_json::from_str::<InstanceRegistry>(&content) {
                    entries.push(registry);
                }
            }
        }
    }
    Ok(entries)
}

/// Check if an instance's socket is alive.
pub fn is_instance_healthy(registry: &InstanceRegistry) -> bool {
    let socket_path = Path::new(&registry.socket_path);
    socket_path.exists() && is_socket_alive(socket_path)
}

/// Remove stale registry entries whose sockets are gone.
pub fn prune_stale() -> Result<usize> {
    let entries = list_all()?;
    let mut pruned = 0;
    for entry in &entries {
        if !is_instance_healthy(entry) {
            cleanup(&entry.instance_id);
            pruned += 1;
        }
    }
    Ok(pruned)
}
