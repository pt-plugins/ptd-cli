use anyhow::{Context, Result};
use clap::Args;

use crate::shared::constants::NATIVE_HOST_NAME;
use crate::shared::paths::BrowserFamily;

#[derive(Args)]
pub struct InstallArgs {
    /// Target browser family
    #[arg(long)]
    pub browser: BrowserFamily,

    /// Extension ID for Chrome-family browsers.
    /// Defaults to the official Web Store / Add-ons ID for the target browser.
    /// Can be specified multiple times to allow additional IDs
    /// (e.g. for unpacked development builds).
    #[arg(long)]
    pub extension_id: Vec<String>,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let configured_host = std::env::var_os("PTD_NATIVE_HOST_PATH").map(std::path::PathBuf::from);
    let host_binary = if let Some(path) = configured_host {
        if !path.is_absolute() {
            anyhow::bail!("PTD_NATIVE_HOST_PATH must be an absolute path");
        }
        path
    } else {
        std::env::current_exe()
            .context("cannot determine current executable path")?
            .parent()
            .context("executable has no parent directory")?
            .join(if cfg!(windows) { "ptd-host.exe" } else { "ptd-host" })
    };

    if !host_binary.exists() {
        anyhow::bail!(
            "ptd-host binary not found at {}. Make sure both binaries are in the same directory.",
            host_binary.display()
        );
    }

    // Preserve an explicitly configured path: package managers can point this
    // at a stable symlink that survives upgrades. For standalone installs,
    // canonicalize the binary next to `ptd` as before.
    let host_path = if std::env::var_os("PTD_NATIVE_HOST_PATH").is_some() {
        host_binary
    } else {
        dunce::canonicalize(&host_binary).context("failed to resolve ptd-host path")?
    };

    let manifest = if args.browser.is_firefox() {
        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_extensions": ["ptdepiler.ptplugins@gmail.com"]
        })
    } else {
        // Official extension IDs from the Chrome Web Store / Edge Add-ons.
        let default_ids: &[&str] = match args.browser {
            BrowserFamily::Chrome => &["iloddidemhbedaopmipajgclofjocogb"],
            BrowserFamily::Edge => &["kbijhmckhndmeckonoikakdfdlbnlkde"],
            BrowserFamily::Chromium => &["iloddidemhbedaopmipajgclofjocogb"],
            _ => &[],
        };

        let ids: Vec<&str> = if args.extension_id.is_empty() {
            default_ids.to_vec()
        } else {
            // User-specified IDs plus the official defaults.
            let mut combined: Vec<&str> = args.extension_id.iter().map(|s| s.as_str()).collect();
            for id in default_ids {
                if !combined.contains(id) {
                    combined.push(id);
                }
            }
            combined
        };

        let origins: Vec<String> = ids
            .iter()
            .map(|id| format!("chrome-extension://{id}/"))
            .collect();

        // If a manifest already exists, merge in any existing allowed_origins
        // so that installing for one browser doesn't break another.
        let manifest_path = args.browser.native_host_manifest_path();
        let mut all_origins = origins.clone();
        if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(existing) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(arr) = existing.get("allowed_origins").and_then(|v| v.as_array()) {
                        for v in arr {
                            if let Some(s) = v.as_str() {
                                if !all_origins.contains(&s.to_string()) {
                                    all_origins.push(s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_origins": all_origins
        })
    };

    let manifest_path = args.browser.native_host_manifest_path();
    let manifest_dir = manifest_path.parent().unwrap();

    std::fs::create_dir_all(manifest_dir)
        .with_context(|| format!("failed to create directory {}", manifest_dir.display()))?;

    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, &json)
        .with_context(|| format!("failed to write manifest to {}", manifest_path.display()))?;

    println!("Native messaging host manifest installed:");
    println!("  Path: {}", manifest_path.display());
    println!("  Host binary: {}", host_path.display());

    // On Windows, also write registry key pointing to the manifest
    #[cfg(target_os = "windows")]
    {
        write_windows_registry(&args.browser, &manifest_path)?;
    }

    println!();
    println!("Restart your browser or reload the PT-Depiler extension to activate.");

    Ok(())
}

/// Write a Windows registry key under HKCU pointing to the native messaging host manifest.
///
/// Each browser family has its own registry path:
/// - Chrome:   HKCU\Software\Google\Chrome\NativeMessagingHosts\<name>
/// - Chromium: HKCU\Software\Chromium\NativeMessagingHosts\<name>
/// - Edge:     HKCU\Software\Microsoft\Edge\NativeMessagingHosts\<name>
/// - Firefox:  HKCU\Software\Mozilla\NativeMessagingHosts\<name>
#[cfg(target_os = "windows")]
fn write_windows_registry(
    browser: &BrowserFamily,
    manifest_path: &std::path::Path,
) -> Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let reg_path = match browser {
        BrowserFamily::Chrome => {
            format!(r"Software\Google\Chrome\NativeMessagingHosts\{NATIVE_HOST_NAME}")
        }
        BrowserFamily::Chromium => {
            format!(r"Software\Chromium\NativeMessagingHosts\{NATIVE_HOST_NAME}")
        }
        BrowserFamily::Edge => {
            format!(r"Software\Microsoft\Edge\NativeMessagingHosts\{NATIVE_HOST_NAME}")
        }
        BrowserFamily::Firefox => {
            format!(r"Software\Mozilla\NativeMessagingHosts\{NATIVE_HOST_NAME}")
        }
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(&reg_path)
        .with_context(|| format!("failed to create registry key: HKCU\\{reg_path}"))?;

    key.set_value("", &manifest_path.to_string_lossy().to_string())
        .with_context(|| format!("failed to set registry value for HKCU\\{reg_path}"))?;

    println!("  Registry: HKCU\\{reg_path}");

    Ok(())
}
