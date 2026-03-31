use anyhow::Result;
use clap::Args;

use crate::shared::paths::BrowserFamily;

#[derive(Args)]
pub struct UninstallArgs {
    /// Target browser family
    #[arg(long)]
    pub browser: BrowserFamily,
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let manifest_path = args.browser.native_host_manifest_path();

    if manifest_path.exists() {
        std::fs::remove_file(&manifest_path)?;
        println!("Removed: {}", manifest_path.display());
    } else {
        println!("No manifest found at {}", manifest_path.display());
    }

    #[cfg(target_os = "windows")]
    {
        remove_windows_registry(&args.browser);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn remove_windows_registry(browser: &BrowserFamily) {
    use crate::shared::constants::NATIVE_HOST_NAME;
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
    match hkcu.delete_subkey(&reg_path) {
        Ok(()) => println!("Removed registry key: HKCU\\{reg_path}"),
        Err(_) => println!("No registry key found at HKCU\\{reg_path}"),
    }
}
