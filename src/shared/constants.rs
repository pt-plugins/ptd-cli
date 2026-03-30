/// Native messaging host name registered with the browser.
pub const NATIVE_HOST_NAME: &str = "com.ptd.native";

/// Default timeout for CLI requests in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Name of the directory under ~/.ptd/ that holds instance sockets and registry files.
pub const INSTANCES_DIR: &str = "instances";

/// Name of the directory under ~/.ptd/ that holds per-instance logs.
pub const LOGS_DIR: &str = "logs";

/// Name of the directory under ~/.ptd/ that holds per-instance caches.
pub const CACHE_DIR: &str = "cache";

/// Hello handshake timeout in seconds. Daemon exits if no hello arrives.
pub const HELLO_TIMEOUT_SECS: u64 = 5;

/// Methods the extension bridge will accept from the CLI.
pub const ALLOWED_METHODS: &[&str] = &[
    // Storage and logging (read-only)
    "getExtStorage",
    "getLogger",
    // Site config
    "getSiteUserConfig",
    "getSiteFavicon",
    "clearSiteFaviconCache",
    // Search
    "getSiteSearchResult",
    "getMediaServerSearchResult",
    // Download and downloader
    "getDownloaderConfig",
    "getDownloaderVersion",
    "getDownloaderStatus",
    "getTorrentDownloadLink",
    "getTorrentInfoForVerification",
    "downloadTorrent",
    "getDownloadHistory",
    "getDownloadHistoryById",
    "deleteDownloadHistoryById",
    "clearDownloadHistory",
    // User info
    "getSiteUserInfoResult",
    "cancelUserInfoQueue",
    "getSiteUserInfo",
    "removeSiteUserInfo",
    // Keep-upload
    "getKeepUploadTasks",
    "getKeepUploadTaskById",
    "createKeepUploadTask",
    "updateKeepUploadTask",
    "deleteKeepUploadTask",
    "clearKeepUploadTasks",
];
