# Source Setup Examples

Example source configurations for various use cases. All sources are configured through the UmbraRelay UI.

## Minimal Setup

**RSS Feed Only**:
1. Add RSS feed: "Hacker News"
2. URL: `https://news.ycombinator.com/rss`
3. Poll interval: 10m (default)

## GitHub-Only Setup

**Track GitHub Issues/PRs**:
1. Add GitHub source: "My Project"
2. Owner: `dev-syndicate`
3. Repository: `UmbraRelay`
4. Assigned Only: Enabled
5. Poll interval: 5m

## Mixed Sources Setup

**Combine GitHub and RSS**:
1. Add GitHub source: "Main Project"
   - Owner: `dev-syndicate`
   - Repository: `UmbraRelay`
   - Assigned Only: Enabled
   - Poll interval: 5m

2. Add GitHub source: "Side Project"
   - Owner: `rust-lang`
   - Repository: `rust`
   - Assigned Only: Disabled
   - Poll interval: 5m

3. Add RSS feed: "Hacker News"
   - URL: `https://news.ycombinator.com/rss`
   - Poll interval: 10m

4. Add RSS feed: "Lobsters"
   - URL: `https://lobste.rs/rss`
   - Poll interval: 15m

## Developer Workflow

**Frequent GitHub Updates**:
1. Add GitHub sources for your active projects
2. Set poll interval to 3-5 minutes
3. Enable "Assigned Only" to reduce noise
4. Add RSS feed for tech news (30m interval)

## News Aggregation

**Multiple RSS Feeds**:
1. Add RSS feed: "Hacker News" (15m interval)
2. Add RSS feed: "Lobsters" (15m interval)
3. Add RSS feed: "TechCrunch" (30m interval)
4. Add RSS feed: "Dev.to" (30m interval)

## Poll Interval Guidelines

- **GitHub**: 5-10 minutes (respects rate limits)
- **RSS News**: 15-30 minutes
- **RSS Blogs**: 30-60 minutes
- **Fast Updates**: 5 minutes (use sparingly)

## Best Practices

1. **Start Simple**: Begin with 1-2 sources
2. **Test Intervals**: Find optimal poll frequencies
3. **Respect APIs**: Don't poll too frequently
4. **Descriptive Names**: Use clear source names
5. **Monitor Performance**: Adjust based on app performance

## Tips

- Use "Assigned Only" for GitHub to reduce noise
- Adjust poll intervals based on source update frequency
- Disable sources you're not actively monitoring
- Use manual sync for on-demand updates
