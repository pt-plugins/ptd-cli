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

### Build from source

```bash
cargo build --release
# Produces target/release/ptd and target/release/ptd-host
```

### Register the native messaging host

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

After installing, reload the PT-Depiler extension. Verify with:

```bash
ptd status
```

## Usage

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
