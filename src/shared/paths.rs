use std::path::PathBuf;

use crate::shared::constants::{CACHE_DIR, INSTANCES_DIR, LOGS_DIR, NATIVE_HOST_NAME};

/// Root directory: ~/.ptd/
pub fn ptd_home() -> PathBuf {
    dirs::home_dir()
        .expect("cannot determine home directory")
        .join(".ptd")
}

/// Directory containing instance sockets and registry JSON files.
pub fn instances_dir() -> PathBuf {
    ptd_home().join(INSTANCES_DIR)
}

/// Path to an instance's Unix domain socket.
pub fn instance_socket_path(instance_id: &str) -> PathBuf {
    instances_dir().join(format!("{instance_id}.sock"))
}

/// Path to an instance's registry metadata JSON file.
pub fn instance_registry_path(instance_id: &str) -> PathBuf {
    instances_dir().join(format!("{instance_id}.json"))
}

/// Directory for per-instance logs.
pub fn logs_dir() -> PathBuf {
    ptd_home().join(LOGS_DIR)
}

/// Path to a specific instance's log file.
pub fn instance_log_path(instance_id: &str) -> PathBuf {
    logs_dir().join(format!("{instance_id}.log"))
}

/// Directory for per-instance caches (search results, etc.).
pub fn cache_dir(instance_id: &str) -> PathBuf {
    ptd_home().join(CACHE_DIR).join(instance_id)
}

/// Path to the last search results cache for a given instance.
pub fn last_search_path(instance_id: &str) -> PathBuf {
    cache_dir(instance_id).join("last-search.json")
}

/// Supported browser families for native messaging host manifest installation.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum BrowserFamily {
    Chrome,
    Chromium,
    Edge,
    Firefox,
}

impl BrowserFamily {
    /// Returns the directory where the native messaging host manifest should be placed.
    pub fn native_host_manifest_dir(&self) -> PathBuf {
        let home = dirs::home_dir().expect("cannot determine home directory");

        #[cfg(target_os = "linux")]
        match self {
            BrowserFamily::Chrome => home.join(".config/google-chrome/NativeMessagingHosts"),
            BrowserFamily::Chromium => home.join(".config/chromium/NativeMessagingHosts"),
            BrowserFamily::Edge => home.join(".config/microsoft-edge/NativeMessagingHosts"),
            BrowserFamily::Firefox => home.join(".mozilla/native-messaging-hosts"),
        }

        #[cfg(target_os = "macos")]
        match self {
            BrowserFamily::Chrome => home.join("Library/Application Support/Google/Chrome/NativeMessagingHosts"),
            BrowserFamily::Chromium => home.join("Library/Application Support/Chromium/NativeMessagingHosts"),
            BrowserFamily::Edge => home.join("Library/Application Support/Microsoft Edge/NativeMessagingHosts"),
            BrowserFamily::Firefox => home.join("Library/Application Support/Mozilla/NativeMessagingHosts"),
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = dirs::data_dir().expect("cannot determine AppData directory");
            match self {
                BrowserFamily::Firefox => appdata.join("Mozilla").join("NativeMessagingHosts"),
                _ => appdata.join("PTDepiler").join("NativeMessagingHosts"),
            }
        }
    }

    /// Returns the full path to the native messaging host manifest file.
    pub fn native_host_manifest_path(&self) -> PathBuf {
        self.native_host_manifest_dir().join(format!("{NATIVE_HOST_NAME}.json"))
    }

    /// Whether this browser uses Firefox-style `allowed_extensions` instead of `allowed_origins`.
    pub fn is_firefox(&self) -> bool {
        matches!(self, BrowserFamily::Firefox)
    }
}
