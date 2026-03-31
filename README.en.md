# ptd-cli

Command-line interface for the [PT-Depiler](https://github.com/pt-plugins/PT-depiler) browser extension via Chrome Native Messaging.

Search torrents, manage downloads, query user info, and manage cross-seeding tasks from the terminal — all operations execute through the running browser extension, reusing its cookies, site definitions, and downloader configurations.

## Architecture

```
ptd search "avatar" --site chdbits
  |
  v
Unix socket --> ptd-host daemon --stdout--> Chrome --> Extension bridge
                                                          |
                                                    sendMessage("getSiteSearchResult", {...})
                                                          |
                                                    offscreen handlers (HTTP + DOM parsing)
                                                          |
CLI <-- Unix socket <-- ptd-host <--stdin-- Chrome <------+
```

Three components:

- **`ptd`** — CLI client. Discovers running browser instances, connects via Unix socket, sends commands, prints results.
- **`ptd-host`** — Native messaging host daemon. Chrome spawns one per browser profile. Bridges CLI requests to the extension and routes responses back.
- **Extension bridge** — Small addition to PT-Depiler's background script. Dispatches CLI requests through the existing `sendMessage()` system.

## Install

### 1. Download pre-built binaries

Download the latest `ptd` and `ptd-host` from [GitHub Releases](https://github.com/pt-plugins/ptd-cli/releases), extract them into the same directory, and add it to your `PATH`.

> **AI Agent users:** Download pre-built binaries from the Release page instead of building from source.

<details>
<summary>Build from source</summary>

```bash
cargo build --release
# Produces target/release/ptd and target/release/ptd-host
```

Place both `ptd` and `ptd-host` in the same directory, and add it to your `PATH`.

</details>

### 2. Register the native messaging host

> **Important:** Complete this step **before** installing or enabling the PT-Depiler extension.
> Chrome only reads native messaging host registrations at startup.
> If you register the host while Chrome is already running, you must **fully quit Chrome** (including background processes) and relaunch it.
> On Windows, use `taskkill /f /im chrome.exe` or check the system tray — simply closing the window is not enough if "Continue running background apps when Google Chrome is closed" is enabled.

```bash
# Chrome
ptd install --browser chrome --extension-id <YOUR_EXTENSION_ID>

# Firefox
ptd install --browser firefox

# Chromium / Edge
ptd install --browser chromium --extension-id <ID>
ptd install --browser edge --extension-id <ID>
```

Find your extension ID at `chrome://extensions` with Developer Mode enabled.

### 3. Enable in extension

Open the PT-Depiler extension settings page, go to **Settings > General > Native Bridge** tab:

1. Click **Grant Permission** to enable the `nativeMessaging` permission
2. Toggle **Enable Native Bridge** on
3. Click **Test Connection** to verify

### 4. Verify

```bash
ptd status
# Should show a healthy instance
```

If `ptd status` shows no instances, make sure:
1. The browser is running with the PT-Depiler extension enabled
2. You fully restarted the browser after running `ptd install`
3. The extension has the `nativeMessaging` permission granted and Native Bridge enabled

## Usage

### Discovery

```bash
ptd status                                    # Show running browser instances
ptd site list                                 # List all configured sites (ID, name, URL)
ptd site list --table                         # Table format
ptd downloader list                           # List all downloaders (ID, name, type, address)
ptd downloader list --table                   # Table format
```

> **Tip:** Before running commands that require a site ID or downloader ID, run `ptd site list` and `ptd downloader list` first to discover valid IDs.

### Search

```bash
# Search all configured sites
ptd search "avatar"

# Search specific sites
ptd search "avatar" --site chdbits
ptd search "avatar" --site chdbits --site btschool

# Pretty print results
ptd search "avatar" --site chdbits --pretty

# Advanced search with entry file
ptd search "avatar" --site chdbits --entry-file ./search-config.json
```

### Download

```bash
# Download by index from last search results
ptd download 0 --downloader <downloader-id>

# Download with full option file
ptd download --option-file ./download-option.json
```

### Downloader

```bash
ptd downloader status <downloader-id>
ptd downloader config <downloader-id>
ptd downloader version <downloader-id>
```

### User Info

```bash
ptd user-info current <site-id>          # Fetch live user stats
ptd user-info history <site-id>          # View historical data
ptd user-info remove <site-id> <dates>   # Remove entries
ptd user-info cancel                     # Cancel pending fetches
```

### Site Config

```bash
ptd site config <site-id>
ptd site favicon <site-id> [--flush]
```

### Download History

```bash
ptd download-history                     # List all
ptd download-history get <id>            # Get specific entry
ptd download-history delete <id>         # Delete entry
ptd download-history clear               # Clear all
```

### Keep-Upload (Cross-Seeding)

```bash
ptd keep-upload list
ptd keep-upload get <task-id>
ptd keep-upload create --file ./task.json
ptd keep-upload update --file ./task.json
ptd keep-upload delete <task-id>
ptd keep-upload clear
```

### Setup & Discovery

```bash
ptd install --browser chrome --extension-id <id>
ptd uninstall --browser chrome
ptd status
```

## Global Options

```
--instance <id>       Select browser/profile instance (prefix match, e.g. --instance fe4c)
--timeout <seconds>   Request timeout (default: 30)
--format <format>     Output format: json (default), pretty, table
--pretty              Alias for --format pretty
--table               Alias for --format table
```

Environment variable: `PTD_INSTANCE=<id>`

## Multi-Instance Support

If you run multiple browsers or profiles with PT-Depiler, each gets its own daemon and socket. The CLI auto-selects when only one is running:

```bash
$ ptd status
  fe4cb61e [healthy] browser=chrome ext=icblbk... since=2026-03-30T07:20:51Z
  a3d91f02 [healthy] browser=firefox ext=ptdep... since=2026-03-30T08:15:22Z

2 instance(s), 2 healthy

$ ptd --instance fe4c search "test" --site chdbits   # prefix match
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Command failed (extension error, timeout, bad input) |
| 2 | No healthy instance found |
| 3 | Multiple instances, none selected |

## Output

Default output is compact JSON, suitable for piping to `jq`:

```bash
ptd search "test" --site chdbits | jq '.[0].title'
ptd user-info current chdbits | jq '.ratio'
```

## License

MIT
