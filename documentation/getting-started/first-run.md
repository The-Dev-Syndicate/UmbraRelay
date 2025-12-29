# First Run Setup

This guide will help you set up UmbraRelay for the first time.

## Launching UmbraRelay

1. **First Launch**: When you first open UmbraRelay, you'll see an empty inbox
2. **Navigation**: Use the sidebar to switch between "Inbox" and "Sources"
3. **Initial State**: The app is ready to use, but you need to add sources first

## Adding Your First RSS Feed

1. **Navigate to Sources**: Click "Sources" in the sidebar
2. **Select RSS Tab**: The "RSS Feed" tab should be selected by default
3. **Fill in the form**:
   - **Name**: Enter a friendly name (e.g., "Hacker News")
   - **URL**: Enter the RSS feed URL (e.g., `https://news.ycombinator.com/rss`)
   - **Poll Interval**: Optional (default: 10m)
4. **Add Source**: Click "Add RSS Feed"
5. **Sync**: Click the "Sync" button next to your new source to fetch items immediately

## Adding Your First GitHub Source

1. **Navigate to Sources**: Click "Sources" in the sidebar
2. **Select GitHub Tab**: Click the "GitHub" tab
3. **Get a GitHub Token**:
   - Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
   - Click "Generate new token (classic)"
   - Give it a name (e.g., "UmbraRelay")
   - Select scopes: `repo` (for private repos) or `public_repo` (for public repos only)
   - Click "Generate token"
   - **Copy the token immediately** (you won't see it again)
4. **Fill in the form**:
   - **Name**: Enter a friendly name (e.g., "My Project")
   - **Owner**: Enter the GitHub username or organization
   - **Repository**: Enter the repository name
   - **GitHub Token**: Paste your token
   - **Assigned Only**: Check if you only want issues/PRs assigned to you
5. **Add Source**: Click "Add GitHub Source"
6. **Sync**: Click the "Sync" button to fetch items immediately

## Understanding the Inbox

After adding sources and syncing:

1. **Navigate to Inbox**: Click "Inbox" in the sidebar
2. **View Items**: You'll see items from your sources
3. **Item States**:
   - **Unread**: New items (highlighted with blue border)
   - **Read**: Items you've viewed
   - **Archived**: Items you've archived
4. **Filters**: Use the filter buttons to view All, Unread, Read, or Archived items

## Next Steps

- [Basic Usage](basic-usage.md) - Learn how to use the inbox
- [Adding Sources](../user-guide/adding-sources.md) - More details on source configuration
- [Configuration](../user-guide/configuration.md) - Advanced configuration options

## Tips

- **Start Small**: Add one or two sources first to get familiar with the interface
- **Sync Manually**: Use the "Sync" button to fetch items on demand
- **Automatic Polling**: Sources will automatically sync at their configured intervals
- **GitHub Tokens**: Keep your tokens secure - they're stored locally and encrypted

