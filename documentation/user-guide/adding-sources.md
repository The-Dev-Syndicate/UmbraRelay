# Adding Sources

Learn how to add and configure RSS feeds and GitHub sources.

## RSS Feeds

### Adding via UI

1. Navigate to **Sources** in the sidebar
2. Click the **RSS Feed** tab
3. Fill in the form:
   - **Name**: Friendly name for the feed (e.g., "Hacker News")
   - **URL**: RSS feed URL (e.g., `https://news.ycombinator.com/rss`)
   - **Poll Interval**: Optional (default: 10m)
     - Format: `5m`, `10m`, `1h`, etc.
4. Click **Add RSS Feed**

### Finding RSS Feeds

Many websites provide RSS feeds. Look for:
- RSS icon/link in the footer
- `/feed` or `/rss` URL paths
- Feed discovery in browser (some browsers show RSS icon)

## GitHub Sources

### Prerequisites

You need a GitHub Personal Access Token:

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Click **Generate new token (classic)**
3. Give it a name (e.g., "UmbraRelay")
4. Select scopes:
   - `repo` - For private repositories
   - `public_repo` - For public repositories only
5. Click **Generate token**
6. **Copy the token immediately** (you won't see it again)

### Adding via UI

1. Navigate to **Sources** in the sidebar
2. Click the **GitHub** tab
3. Fill in the form:
   - **Name**: Friendly name (e.g., "My Project")
   - **Owner**: GitHub username or organization
   - **Repository**: Repository name
   - **GitHub Token**: Your personal access token
   - **Assigned Only**: Check to only show issues/PRs assigned to you
4. Click **Add GitHub Source**

### Token Security

- Tokens are stored securely using Tauri's secure storage
- Tokens are encrypted and stored locally
- Never share your tokens

### What Gets Fetched

- **Issues**: Open issues assigned to you (if `assigned_only` is enabled)
- **Pull Requests**: Open PRs assigned to you (if `assigned_only` is enabled)
- **All Open Items**: If `assigned_only` is disabled, fetches all open issues/PRs

## Source Management

### Enabling/Disabling

- Toggle the **Enabled** switch to temporarily disable a source
- Disabled sources won't be polled automatically
- Items from disabled sources remain in your inbox

### Manual Sync

- Click **Sync** button next to any source
- Fetches items immediately
- Useful for testing or getting updates on demand

### Editing Sources

- Click **Edit** button to modify source settings
- Update name, URL, poll interval, or other settings as needed

### Removing Sources

- Click **Delete** button
- Confirms before deletion
- Removes source and all associated items

## Poll Intervals

Configure how often sources are checked:

- **Format**: `5m`, `10m`, `30m`, `1h`, `2h`
- **RSS Default**: 10 minutes
- **GitHub Default**: 5 minutes
- **Minimum**: 1 minute (not recommended)
- **Maximum**: No limit (but be respectful of API rate limits)

## Tips

- **Start with Few Sources**: Add 1-2 sources first to test
- **Test with Manual Sync**: Use "Sync" button to test sources
- **Monitor Rate Limits**: GitHub has rate limits (5000 requests/hour for authenticated)
- **RSS Reliability**: Some RSS feeds may be slow or unreliable

## Troubleshooting

- **RSS Not Syncing**: Check URL is valid and accessible
- **GitHub Auth Errors**: Verify token has correct scopes
- **No Items**: Check if source is enabled and has content
- See [Troubleshooting Guide](troubleshooting.md) for more help

