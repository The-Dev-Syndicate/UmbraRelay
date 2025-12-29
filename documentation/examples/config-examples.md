# Configuration Examples

Example TOML configurations for various use cases.

## Minimal RSS-Only Config

```toml
[[rss]]
name = "Hacker News"
url = "https://news.ycombinator.com/rss"
```

## GitHub-Only Config

```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "dev-syndicate"
repo = "UmbraRelay"
assigned_only = true
```

## Mixed Sources Config

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

## Advanced Polling Intervals

```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "user"
repo = "repo"
assigned_only = true

[[rss]]
name = "Fast Feed"
url = "https://example.com/feed.xml"
poll_interval = "5m"  # Check every 5 minutes

[[rss]]
name = "Slow Feed"
url = "https://example.com/slow-feed.xml"
poll_interval = "1h"  # Check every hour
```

## Multiple GitHub Repos

```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "user1"
repo = "project1"
assigned_only = true

[[github.repos]]
owner = "user1"
repo = "project2"
assigned_only = true

[[github.repos]]
owner = "org"
repo = "public-repo"
assigned_only = false
```

## Developer Workflow

```toml
[github]
poll_interval = "3m"  # Check frequently for issues/PRs

[[github.repos]]
owner = "my-org"
repo = "main-project"
assigned_only = true

[[github.repos]]
owner = "my-org"
repo = "side-project"
assigned_only = true

[[rss]]
name = "Tech News"
url = "https://tech-news.example.com/rss"
poll_interval = "30m"  # Less frequent for news
```

## News Aggregation

```toml
[[rss]]
name = "Hacker News"
url = "https://news.ycombinator.com/rss"
poll_interval = "15m"

[[rss]]
name = "Lobsters"
url = "https://lobste.rs/rss"
poll_interval = "15m"

[[rss]]
name = "Reddit Programming"
url = "https://www.reddit.com/r/programming/.rss"
poll_interval = "20m"

[[rss]]
name = "Dev.to"
url = "https://dev.to/feed"
poll_interval = "30m"
```

## Notes

- **GitHub tokens** are NOT stored in TOML (stored securely)
- **Poll intervals** use format: `5m`, `10m`, `1h`, etc.
- **Comments** start with `#`
- **Arrays** use `[[key]]` syntax in TOML

## Best Practices

1. **Start Simple**: Begin with 1-2 sources
2. **Test Intervals**: Find optimal poll intervals
3. **Respect APIs**: Don't poll too frequently
4. **Descriptive Names**: Use clear source names
5. **Backup Config**: Keep config in version control (without tokens)

