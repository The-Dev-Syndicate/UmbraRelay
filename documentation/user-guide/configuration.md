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

## Best Practices

1. **Start Small**: Begin with 1-2 sources to get familiar
2. **Test Intervals**: Find optimal poll intervals for your workflow
3. **Respect APIs**: Don't poll too frequently (especially GitHub)
4. **Descriptive Names**: Use clear, descriptive names for sources
5. **Monitor Rate Limits**: GitHub has rate limits (5000 requests/hour for authenticated)

## Troubleshooting

- **Source Not Syncing**: Check if source is enabled
- **Invalid Poll Interval**: Use format like `5m` or `1h`
- **GitHub Auth Errors**: Verify token has correct scopes
- See [Troubleshooting Guide](troubleshooting.md) for more help
