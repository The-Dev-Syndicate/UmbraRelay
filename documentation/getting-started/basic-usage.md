# Basic Usage

Learn the basics of using UmbraRelay's inbox interface.

## Navigation

- **Inbox**: View and manage your aggregated items
- **Sources**: Add, edit, and manage your RSS feeds and GitHub sources

## Inbox Interface

### Viewing Items

1. **Item List**: The main view shows all items from your sources
2. **Item Card**: Each item displays:
   - Type badge (post, issue, pr)
   - State badge (unread, read, archived)
   - Title
   - Summary (if available)
   - Creation date

### Filtering Items

Use the filter buttons at the top:
- **All**: Show all items regardless of state
- **Unread**: Show only unread items
- **Read**: Show only read items
- **Archived**: Show only archived items

### Viewing Item Details

1. **Click an item** to open the detail view
2. **Item Detail View** shows:
   - Full title
   - Complete summary/description
   - External URL
   - Actions: Mark Read/Unread, Archive, Open Link

### Managing Item States

**Mark as Read**:
- Click an item to view details (automatically marks as read)
- Or use "Mark Read" button in detail view

**Mark as Unread**:
- Use "Mark Unread" button in detail view

**Archive**:
- Use "Archive" button in detail view
- Archived items are hidden from default view (use "Archived" filter)

**Open External Link**:
- Click "Open Link" button to open the original source in your browser

## Source Management

### Viewing Sources

1. Navigate to "Sources" in the sidebar
2. See all your configured sources
3. Each source shows:
   - Name and type
   - Enabled/Disabled status
   - Last sync time
   - Actions: Sync, Edit, Delete

### Enabling/Disabling Sources

- Toggle the "Enabled" switch to temporarily disable a source
- Disabled sources won't be polled automatically

### Manual Sync

- Click "Sync" button next to any source to fetch items immediately
- Useful for testing or getting updates on demand

## Keyboard Shortcuts

(Currently not implemented - coming in future versions)

## Tips

- **Unread Items**: Unread items have a blue left border for easy identification
- **Automatic Polling**: Sources sync automatically at configured intervals
- **State Persistence**: Item states (read/unread/archived) are saved locally
- **No Cloud Sync**: All data stays on your machine

## Next Steps

- [Inbox Management](../user-guide/inbox-management.md) - Advanced inbox features
- [Adding Sources](../user-guide/adding-sources.md) - Detailed source configuration
- [Configuration](../user-guide/configuration.md) - Configuration guide

