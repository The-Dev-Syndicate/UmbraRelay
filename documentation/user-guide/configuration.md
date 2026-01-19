# Configuration

UmbraRelay is configured entirely through the user interface. All settings are stored locally in the SQLite database.

## Source Configuration

Sources are configured when you add them through the UI. See [Adding Sources](adding-sources.md) for detailed instructions.

### RSS Feed Configuration

When adding an RSS feed, you can configure:

- **Name**: A friendly name for the feed
- **URL**: The RSS feed URL
- **Poll Interval**: How often to check for new items (default: 10 minutes)
  - Format: `5m`, `10m`, `30m`, `1h`, `2h`
  - Minimum: 1 minute (not recommended)
  - Maximum: No limit (but be respectful of feed providers)

### GitHub Source Configuration

When adding a GitHub source, you can configure:

- **Name**: A friendly name for the source
- **Owner**: GitHub username or organization
- **Repository**: Repository name
- **GitHub Token**: Personal access token (stored securely)
- **Assigned Only**: Only show issues/PRs assigned to you
- **Poll Interval**: How often to check for new items (default: 5 minutes)

## Poll Interval Format

Poll intervals use a simple time format:

- `5s` - 5 seconds
- `1m` - 1 minute
- `5m` - 5 minutes
- `10m` - 10 minutes
- `30m` - 30 minutes
- `1h` - 1 hour
- `2h` - 2 hours

## GitHub Token Storage

**Important**: GitHub tokens are stored securely:

- Tokens are encrypted using Tauri's secure storage
- Stored locally on your machine
- Never transmitted to external services
- Per-source basis (each source has its own token)

To update a token:
- Use the UI to edit the source
- Or remove and re-add the source with a new token

## Editing Sources

To modify a source's configuration:

1. Navigate to **Sources** in the sidebar
2. Find the source you want to edit
3. Click the **Edit** button
4. Modify any settings (name, URL, poll interval, etc.)
5. Save your changes

Changes take effect immediately.

## Enabling/Disabling Sources

You can temporarily disable sources without deleting them:

- Toggle the **Enabled** switch next to any source
- Disabled sources won't be polled automatically
- Items from disabled sources remain in your inbox
- You can manually sync disabled sources

## Article View Preferences

UmbraRelay can automatically detect and fetch full article content from websites when RSS/Atom feeds only provide summaries.

### Article View Modes

You can configure how articles are displayed in **Settings > Article View**:

#### Auto (detect & fetch full text)
- **Default mode** - Automatically detects partial feeds
- Fetches full articles in the background for feeds that only provide summaries
- Shows a spinner (⏳) next to article titles while fetching
- Displays full content when available, falls back to feed content if extraction fails
- **Best for**: Most users who want complete articles without manual configuration

#### Feed content only
- Always displays content from the RSS/Atom feed
- Never fetches full articles from websites
- **Best for**: Users who prefer feed content or want to save bandwidth

#### Always fetch full article
- Always attempts to fetch and display full article content
- Falls back to feed content if extraction fails
- **Best for**: Users who want full articles even when feeds provide substantial content

### Content Extraction

- **Enable/Disable**: Toggle "Enable content extraction" in Settings
- **When Disabled**: No articles are fetched, even in Auto mode
- **When Enabled**: Articles are fetched automatically based on your view mode preference

### Visual Indicators

When viewing articles, you'll see indicators showing the content source:

- **⏳ Spinner**: Article is being fetched in the background
- **"Full Article" badge**: Content was extracted from the website
- **"From Feed" badge**: Content is from the RSS/Atom feed

### How It Works

1. **Detection**: When items are synced, UmbraRelay analyzes the feed content
2. **Classification**: Items are marked as "full", "partial", or "unknown"
3. **Extraction**: Partial items automatically trigger background fetching
4. **Caching**: Extracted content is cached - each article is only fetched once
5. **Display**: The UI shows the best available content based on your preferences

### Performance

- Extraction happens **asynchronously** - doesn't slow down feed syncing
- Extracted content is **cached permanently** - fast loading on subsequent views
- **No duplicate fetches** - each article URL is only fetched once
- Small delay between extractions to avoid overwhelming servers

## Best Practices

1. **Start Small**: Begin with 1-2 sources to get familiar
2. **Test Intervals**: Find optimal poll intervals for your workflow
3. **Respect APIs**: Don't poll too frequently (especially GitHub)
4. **Descriptive Names**: Use clear, descriptive names for sources
5. **Monitor Rate Limits**: GitHub has rate limits (5000 requests/hour for authenticated)
6. **Article View**: Use "Auto" mode for best experience with partial feeds

## Troubleshooting

- **Source Not Syncing**: Check if source is enabled
- **Invalid Poll Interval**: Use format like `5m` or `1h`
- **GitHub Auth Errors**: Verify token has correct scopes
- **Articles Not Fetching**: Check that "Enable content extraction" is enabled and view mode is set to "Auto" or "Always fetch"
- **Extraction Failures**: Some websites block automated fetching - feed content will be shown instead
- See [Troubleshooting Guide](troubleshooting.md) for more help
