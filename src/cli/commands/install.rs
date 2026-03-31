use anyhow::{Context, Result};
use clap::Args;

use crate::shared::constants::NATIVE_HOST_NAME;
use crate::shared::paths::BrowserFamily;

#[derive(Args)]
pub struct InstallArgs {
    /// Target browser family
    #[arg(long)]
    pub browser: BrowserFamily,

    /// Extension ID (required for Chrome-family browsers)
    #[arg(long)]
    pub extension_id: Option<String>,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let host_binary = std::env::current_exe()
        .context("cannot determine current executable path")?
        .parent()
        .context("executable has no parent directory")?
        .join(if cfg!(windows) { "ptd-host.exe" } else { "ptd-host" });

    if !host_binary.exists() {
        anyhow::bail!(
            "ptd-host binary not found at {}. Make sure both binaries are in the same directory.",
            host_binary.display()
        );
    }

    let host_path = host_binary
        .canonicalize()
        .context("failed to resolve ptd-host path")?;

    let manifest = if args.browser.is_firefox() {
        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_extensions": ["ptdepiler.ptplugins@gmail.com"]
        })
    } else {
        let ext_id = args.extension_id.as_deref().unwrap_or_else(|| {
            eprintln!("--extension-id is required for Chrome-family browsers.");
            eprintln!("Find it at chrome://extensions with Developer Mode enabled.");
            std::process::exit(1);
        });
        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_origins": [format!("chrome-extension://{ext_id}/")]
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
