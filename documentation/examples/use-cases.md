# Use Cases

Real-world usage scenarios for UmbraRelay.

## Developer Workflow

**Scenario**: Track GitHub issues and PRs assigned to you across multiple projects.

**Setup**:
```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "my-org"
repo = "main-project"
assigned_only = true

[[github.repos]]
owner = "my-org"
repo = "side-project"
assigned_only = true
```

**Workflow**:
1. UmbraRelay polls GitHub every 5 minutes
2. New issues/PRs assigned to you appear in inbox
3. Review items in unified inbox
4. Click to view details or open in GitHub
5. Mark as read/archived as you process them

**Benefits**:
- Single inbox for all projects
- No need to check multiple GitHub repos
- Automatic updates
- Local-first (no cloud sync needed)

## News Aggregation

**Scenario**: Aggregate news from multiple RSS feeds.

**Setup**:
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
name = "TechCrunch"
url = "https://techcrunch.com/feed/"
poll_interval = "30m"
```

**Workflow**:
1. UmbraRelay polls feeds at configured intervals
2. New articles appear in inbox
3. Browse headlines and summaries
4. Click to read full articles
5. Archive items you've read

**Benefits**:
- Unified news feed
- No need to visit multiple sites
- Offline access to summaries
- Local storage (no tracking)

## Mixed Workflow

**Scenario**: Combine GitHub notifications with tech news.

**Setup**:
```toml
[github]
poll_interval = "5m"

[[github.repos]]
owner = "my-org"
repo = "project"
assigned_only = true

[[rss]]
name = "Dev.to"
url = "https://dev.to/feed"
poll_interval = "30m"
```

**Workflow**:
1. Check inbox for GitHub issues/PRs
2. Also see latest tech articles
3. Process items by priority
4. Use filters to focus on specific types

**Benefits**:
- Single interface for work and news
- Context switching reduced
- All information in one place

## Best Practices

1. **Start Small**: Begin with 1-2 sources
2. **Test Intervals**: Find optimal poll frequencies
3. **Use Filters**: Filter by state to focus
4. **Regular Review**: Check inbox regularly
5. **Archive Old Items**: Keep inbox manageable
6. **Respect APIs**: Don't poll too frequently

## Tips

- **GitHub**: Use `assigned_only` to reduce noise
- **RSS**: Adjust poll intervals based on feed update frequency
- **States**: Use read/unread/archived to organize workflow
- **Manual Sync**: Use "Sync" button for on-demand updates

