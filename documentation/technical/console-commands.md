# Console Commands

UmbraRelay exposes several commands that can be called from the browser console for power users and developers. These commands allow you to perform advanced operations, testing, and maintenance tasks.

## Accessing the Console

1. Open UmbraRelay
2. Press `F12` (or `Cmd+Option+I` on macOS) to open Developer Tools
3. Navigate to the "Console" tab
4. Use the commands below

## Available Commands

### Database & Maintenance

#### `clear_source_items(sourceName)`

Deletes all items for a specific source by name. Useful for cleaning up unwanted items from a source without deleting the source itself.

**Parameters:**
- `sourceName` (string, required): The exact name of the source (case-sensitive)

**Returns:**
- `number`: The number of items deleted

**Example:**
```javascript
// Clear all items from a GitHub source
const deleted = await invoke('clear_source_items', { sourceName: 'My GitHub Source' });
console.log(`Deleted ${deleted} items`);
```

**Notes:**
- The source name must match exactly (case-sensitive)
- This permanently deletes all items for the source, regardless of state (unread, read, or archived)
- The source itself is not deleted, only its items
- Useful for cleaning up test data, removing unwanted notifications, or resetting a source's items
- After clearing, you may want to sync the source again to fetch fresh items
- Returns `0` if the source name is not found

---

#### `cleanup_old_items(days?)`

Deletes items older than the specified number of days, while preserving archived items.

**Parameters:**
- `days` (optional, number): Number of days to keep items. Defaults to `30` if not provided.

**Returns:**
- `number`: The number of items deleted

**Example:**
```javascript
// Delete items older than 30 days (default)
await invoke('cleanup_old_items');

// Delete items older than 60 days
await invoke('cleanup_old_items', { days: 60 });
```

**Notes:**
- Archived items are **never** deleted, regardless of age
- Only items with `state != 'archived'` are affected
- This is a permanent operation - deleted items cannot be recovered

---

### Testing & Development

#### `make_items_leaving_soon(count?)`

Updates the `created_at` timestamp of items to make them appear in the "Leaving Soon" view. Useful for testing the "Leaving Soon" feature without waiting 23+ days.

**Parameters:**
- `count` (optional, number): Number of items to update. Defaults to `7` if not provided.

**Returns:**
- `number`: The number of items updated

**Example:**
```javascript
// Update 7 items to appear in "Leaving Soon" (default)
await invoke('make_items_leaving_soon');

// Update 10 items
await invoke('make_items_leaving_soon', { count: 10 });
```

**How it works:**
- Finds the first N non-archived items (ordered by ID)
- Sets their `created_at` timestamps to 23-29 days ago
- Items will appear in "Leaving Soon" with color-coded chips:
  - 7 days left: Green
  - 6 days left: Lighter green
  - 5 days left: Blue
  - 4 days left: Dimming blue
  - 3 days left: Orange
  - 2 days left: Darker orange
  - 1 day left: RED

**Notes:**
- Only updates non-archived items
- Items are distributed across the 7-day range (23-29 days old)
- After running, refresh the "Leaving Soon" view to see changes
- This is primarily for testing - use with caution in production

---

### Item Management

#### `get_items(stateFilter?, groupFilter?)`

Retrieves items from the database with optional filtering.

**Parameters:**
- `stateFilter` (optional, string): Filter by state (`'unread'`, `'read'`, `'archived'`)
- `groupFilter` (optional, string): Filter by group name

**Returns:**
- `Array<Item>`: Array of item objects

**Example:**
```javascript
// Get all items
await invoke('get_items');

// Get only unread items
await invoke('get_items', { stateFilter: 'unread' });

// Get items from a specific group
await invoke('get_items', { groupFilter: 'tech' });
```

---

#### `get_item(id)`

Retrieves a single item by ID.

**Parameters:**
- `id` (number): The item ID

**Returns:**
- `Item`: The item object

**Example:**
```javascript
await invoke('get_item', { id: 123 });
```

---

#### `update_item_state(id, state)`

Updates the state of an item.

**Parameters:**
- `id` (number): The item ID
- `state` (string): New state (`'unread'`, `'read'`, `'archived'`)

**Returns:**
- `void`

**Example:**
```javascript
// Mark item as read
await invoke('update_item_state', { id: 123, state: 'read' });

// Archive an item
await invoke('update_item_state', { id: 123, state: 'archived' });
```

---

### Source Management

#### `get_sources()`

Retrieves all configured sources.

**Returns:**
- `Array<Source>`: Array of source objects

**Example:**
```javascript
const sources = await invoke('get_sources');
console.log(sources);
```

---

#### `sync_source(id)`

Manually triggers a sync for a specific source.

**Parameters:**
- `id` (number): The source ID

**Returns:**
- `void`

**Example:**
```javascript
// Sync source with ID 1
await invoke('sync_source', { id: 1 });
```

---

## Using Commands in the Console

### Basic Usage

All commands use Tauri's `invoke` function. In the browser console, you can access it via:

```javascript
// If using Tauri v2
import { invoke } from '@tauri-apps/api/core';
await invoke('command_name', { param1: value1 });
```

Or if the app exposes it globally:

```javascript
// Check if available
window.__TAURI_INTERNALS__?.invoke('command_name', { param1: value1 });
```

### Helper Function

You can create a helper function in the console for easier access:

```javascript
// Create a helper
const cmd = async (name, args = {}) => {
  const { invoke } = await import('@tauri-apps/api/core');
  return await invoke(name, args);
};

// Use it
await cmd('make_items_leaving_soon', { count: 7 });
await cmd('cleanup_old_items', { days: 30 });
await cmd('clear_source_items', { sourceName: 'My GitHub Source' });
```

### Example Workflow

```javascript
// 1. Check current items
const items = await invoke('get_items');
console.log(`Total items: ${items.length}`);

// 2. Update 7 items to test "Leaving Soon" view
await invoke('make_items_leaving_soon', { count: 7 });

// 3. Refresh the view (or reload the app)
location.reload();

// 4. Clean up old items (keep last 30 days)
const deleted = await invoke('cleanup_old_items', { days: 30 });
console.log(`Deleted ${deleted} old items`);

// 5. Clear all items from a specific source
const cleared = await invoke('clear_source_items', { sourceName: 'My GitHub Source' });
console.log(`Cleared ${cleared} items from source`);
```

## Safety Notes

⚠️ **Warning**: Some commands modify data permanently:

- `cleanup_old_items`: Permanently deletes items (except archived)
- `clear_source_items`: Permanently deletes all items for a source
- `make_items_leaving_soon`: Modifies item timestamps (for testing only)
- `update_item_state`: Changes item states
- `remove_source`: Deletes sources and all their items

Always test commands in a development environment before using in production.

## Troubleshooting

### Command not found

If you get an error that a command doesn't exist:
1. Make sure you're using the correct command name (case-sensitive)
2. Check that the app is running the latest version
3. Verify the command is registered in `src-tauri/src/lib.rs`

### Permission errors

Some commands may require specific permissions. Check the browser console for detailed error messages.

### Database locked

If you see database lock errors:
1. Close any other instances of UmbraRelay
2. Wait a few seconds and try again
3. Restart the app if the issue persists

## Contributing

Want to add more console commands? See the [Development Guide](./development.md) for details on:
- Adding new Tauri commands
- Registering commands in the app
- Testing commands

