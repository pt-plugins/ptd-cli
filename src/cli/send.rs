use std::path::Path;

use anyhow::Result;

use crate::cli::client;
use crate::cli::discovery;
use crate::cli::output::{self, OutputFormat};
use crate::shared::protocol::ResponseMessage;

/// Discover instance, send a request, handle the response, and print the result.
pub fn send_and_print(
    instance: Option<&str>,
    timeout: u64,
    format: OutputFormat,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let registry = discovery::select_instance(instance)?;
    let socket_path_str = registry.socket_path.clone();
    let socket_path = Path::new(&socket_path_str);

    let rt = tokio::runtime::Runtime::new()?;
    let response: ResponseMessage =
        rt.block_on(client::send_request(socket_path, method, params, timeout))?;

    if let Some(error) = response.error {
        anyhow::bail!("[{}] {}", error.code, error.message);
    }

    let result = response.result.unwrap_or(serde_json::Value::Null);
    output::print_value(&result, format)?;
    Ok(result)
}

/// Like send_and_print but returns the raw response without printing.
pub fn send_raw(
    instance: Option<&str>,
    timeout: u64,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let registry = discovery::select_instance(instance)?;
    let socket_path_str = registry.socket_path.clone();
    let socket_path = Path::new(&socket_path_str);

    let rt = tokio::runtime::Runtime::new()?;
    let response: ResponseMessage =
        rt.block_on(client::send_request(socket_path, method, params, timeout))?;

    if let Some(error) = response.error {
        anyhow::bail!("[{}] {}", error.code, error.message);
    }

    Ok(response.result.unwrap_or(serde_json::Value::Null))
}

/// Get the selected instance ID (for cache paths, etc.).
pub fn resolve_instance_id(instance: Option<&str>) -> Result<String> {
    let registry = discovery::select_instance(instance)?;
    Ok(registry.instance_id)
}
