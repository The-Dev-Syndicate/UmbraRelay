# Configuration

UmbraRelay can be configured via TOML file or UI. This guide covers both methods.

## Config File Location

By default, UmbraRelay stores its configuration at:
- **All platforms**: `~/.config/umbrarelay/config.toml`

The path supports `~` expansion for your home directory.

### Custom Config Location

You can override the config file location by setting the `UMBRARELAY_CONFIG_PATH` environment variable:

```bash
# Example: Use a custom location
export UMBRARELAY_CONFIG_PATH="$HOME/.config/umbrarelay/config.toml"

# Or use an absolute path
export UMBRARELAY_CONFIG_PATH="/path/to/my/config.toml"

# Supports ~ expansion
export UMBRARELAY_CONFIG_PATH="~/.my-custom-config/umbrarelay.toml"
```

**Note**: The database will still be stored in the default app data directory, only the config file location is affected.

## TOML Syntax Basics

TOML (Tom's Obvious Minimal Language) is a simple configuration format:

```toml
# Comments start with #

[key]
value = "string"
number = 42
boolean = true

[[array]]
item = "value"
```

## Config Structure

### Global GitHub Settings

```toml
[github]
poll_interval = "5m"  # Default poll interval for all GitHub sources
```

### GitHub Repositories

```toml
[[github.repos]]
owner = "username"
repo = "repository-name"
assigned_only = true  # Only show issues/PRs assigned to you
```

### RSS Feeds

```toml
[[rss]]
name = "Feed Name"
url = "https://example.com/feed.xml"
poll_interval = "10m"  # Optional, defaults to 10m
```

## Complete Example

```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "dev-syndicate"
repo = "UmbraRelay"
assigned_only = true

[[github.repos]]
owner = "rust-lang"
repo = "rust"
assigned_only = false

[[rss]]
name = "Hacker News"
url = "https://news.ycombinator.com/rss"
poll_interval = "10m"

[[rss]]
name = "Lobsters"
url = "https://lobste.rs/rss"
poll_interval = "15m"
```

## UI vs TOML Configuration

### UI Configuration

- **Easier**: No need to edit files
- **Immediate**: Changes take effect immediately
- **Limited**: Some advanced options may require TOML

### TOML Configuration

- **Powerful**: Full control over configuration
- **Version Control**: Can be tracked in git
- **Requires Restart**: Changes require app restart or config reload

## Poll Interval Format

Poll intervals use a simple format:

- `5s` - 5 seconds
- `1m` - 1 minute
- `5m` - 5 minutes
- `10m` - 10 minutes
- `30m` - 30 minutes
- `1h` - 1 hour
- `2h` - 2 hours

## Reloading Configuration

Currently, configuration changes require:
1. Save the TOML file
2. Restart UmbraRelay

Future versions may support hot-reload.

## GitHub Token Storage

**Important**: GitHub tokens are NOT stored in the TOML file for security.

Tokens are stored:
- Securely using Tauri's secure storage
- Encrypted on disk
- Per-source basis

To update a token:
- Use the UI to edit the source
- Or remove and re-add the source

## Best Practices

1. **Backup Config**: Keep a backup of your config file
2. **Version Control**: Track config in git (but exclude tokens)
3. **Test Changes**: Test config changes with one source first
4. **Poll Intervals**: Be respectful of API rate limits
5. **Descriptive Names**: Use clear names for sources

## Troubleshooting

- **Config Not Loading**: Check file syntax and location
- **Invalid Poll Interval**: Use format like `5m` or `1h`
- **Missing Sources**: Verify TOML syntax is correct
- See [Troubleshooting Guide](troubleshooting.md) for more help

## Examples

See [Config Examples](../examples/config-examples.md) for more examples.

