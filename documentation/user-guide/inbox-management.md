# Inbox Management

Learn how to effectively manage your inbox items in UmbraRelay.

## Item States

UmbraRelay uses three item states:

- **Unread**: New items that haven't been viewed
- **Read**: Items you've viewed or marked as read
- **Archived**: Items you've archived (hidden from default view)

## Filtering Items

Use the filter buttons at the top of the inbox:

- **All**: Shows all items regardless of state
- **Unread**: Shows only unread items (default view)
- **Read**: Shows only read items
- **Archived**: Shows only archived items

## Viewing Item Details

1. **Click any item** in the inbox list
2. The detail view shows:
   - Full title (with content source indicator)
   - Complete article content (from feed or extracted)
   - External URL
   - Item metadata (type, state, date)

### Content Source Indicators

When viewing articles, you may see indicators showing where the content comes from:

- **⏳ Spinner**: Article is being fetched in the background (full content loading)
- **"Full Article" badge**: Content was extracted from the website
- **"From Feed" badge**: Content is from the RSS/Atom feed

Articles with partial feeds will automatically fetch full content in the background. See [Configuration](configuration.md#article-view-preferences) for more details.

## Changing Item States

### Mark as Read

- **Automatic**: Clicking an item automatically marks it as read
- **Manual**: Use "Mark Read" button in detail view

### Mark as Unread

- Use "Mark Unread" button in detail view
- Useful for items you want to revisit

### Archive

- Use "Archive" button in detail view
- Archived items are hidden from default view
- Use "Archived" filter to view them

## Opening External Links

- Click "Open Link" button in detail view
- Opens the original source in your default browser
- Useful for reading full articles or viewing GitHub issues/PRs

## Item Types

Items are categorized by type:

- **post**: RSS feed items
- **issue**: GitHub issues
- **pr**: GitHub pull requests

## Tips

- **Unread Indicator**: Unread items have a blue left border
- **State Persistence**: States are saved locally and persist across sessions
- **No Bulk Operations**: Currently, items must be managed individually
- **Automatic Updates**: Items update when sources sync (title/summary may change)

## Best Practices

1. **Regular Review**: Check unread items regularly
2. **Archive Old Items**: Archive items you've processed
3. **Use Filters**: Use filters to focus on specific states
4. **Sync Sources**: Manually sync sources when you need fresh content

