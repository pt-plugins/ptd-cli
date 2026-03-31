---
name: ptd-cli
description: Use when the user asks to search torrents, download torrents, check PT site user info or stats, manage downloaders, query cross-seeding tasks, or interact with the PT-Depiler browser extension from the terminal
---

# ptd-cli

CLI for the PT-Depiler browser extension via Chrome Native Messaging. All operations execute through the running browser extension, reusing its cookies, site definitions, and downloader configurations.

## Installation

If `ptd` is not installed, download pre-built binaries from [GitHub Releases](https://github.com/pt-plugins/ptd-cli/releases). Do NOT build from source — use the release binaries.

## Before you start

Run `ptd status` to confirm a healthy connection to the browser extension. If it fails, the user needs to ensure the browser is running with PT-Depiler loaded and the native host registered (`ptd install`).

## CRITICAL: Always discover site IDs and downloader IDs first

**NEVER guess site IDs or downloader IDs.** They are internal identifiers that often don't match the site's display name (e.g., PTerClub = `pter`, not `pterclub`; M-Team = `mteam`, not `m-team`).

Before performing ANY site-specific or downloader-specific operation, you MUST first retrieve the available IDs:

```bash
ptd site list --table                         # List all site IDs
ptd downloader list --table                   # List all downloader IDs
```

Site IDs are lowercase strings like `chdbits`, `mteam`, `hdhome`.
Downloader IDs are opaque keys like `6JsFPshE1tXYVUVmh_ZL_`, not human names.

**Never guess IDs.** Always list first, then use the exact ID from the output.

## Commands

### Discovery

```bash
ptd status                                    # Running browser instances
ptd site list                                 # All configured sites (id, name, url)
ptd site list --table                         # Table format
ptd downloader list                           # All downloaders (id, name, type, address)
ptd downloader list --table                   # Table format
```

### Search

```bash
ptd search "keyword"                          # All configured sites
ptd search "keyword" --site <site-id>         # Specific site
ptd search "keyword" --site a --site b        # Multiple sites
ptd search "keyword" --pretty                 # Human-readable output
```

Results are cached for `ptd download <index>`.

### Download

```bash
ptd download 0 --downloader <downloader-id>   # By index from last search
ptd download --option-file ./dl.json           # Full option payload
```

The downloader ID is an internal key (e.g. `6JsFPshE1tXYVUVmh_ZL_`), not the human name.

### User Info

```bash
ptd user-info current <site-id>               # Live stats (ratio, bonus, etc.)
ptd user-info history <site-id>               # Historical snapshots
```

### Downloader

```bash
ptd downloader status <id>                    # dl/up speed
ptd downloader config <id>                    # Full config (address, type, etc.)
ptd downloader version <id>
```

### Other

```bash
ptd site config <site-id>                     # Site settings
ptd download-history                          # List all download history
ptd keep-upload list                          # Cross-seeding tasks
```

## Global Options

```
--instance <id>    Select instance (prefix match). Env: PTD_INSTANCE
--timeout <secs>   Default 30
--pretty           Human-readable JSON
--table            Table format for lists
```

Default output is compact JSON, pipe to `jq` for filtering:

```bash
ptd user-info current <site-id> | jq '.ratio'
```

## Exit Codes

- 0: success
- 1: command failed
- 2: no healthy instance (browser not running or extension not loaded)
- 3: multiple instances, use `--instance` to select

## Key Patterns

- **Always list first**: run `ptd site list` / `ptd downloader list` before any command needing IDs
- **Cross-site search**: omit `--site` to search all configured sites
- **Download workflow**: `ptd downloader list` → `ptd search` → pick index → `ptd download` with downloader ID
- **Instance auto-select**: works automatically with one browser; use `--instance` prefix match with multiple
- **Extension must be initialized**: open the extension options page at least once to populate site/downloader config
