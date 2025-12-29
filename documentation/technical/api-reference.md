# API Reference

Complete reference for Tauri command handlers (frontend-backend communication).

## Overview

All commands are async and return `Result<T, String>` where errors are string messages.

## Item Commands

### get_items

Get inbox items with optional state filter.

**Request**:
```typescript
invoke('get_items', { stateFilter?: string })
```

**Parameters**:
- `stateFilter` (optional): `'unread'` | `'read'` | `'archived'`

**Response**:
```typescript
Item[]
```

**Example**:
```typescript
// Get all items
const items = await invoke<Item[]>('get_items');

// Get only unread items
const unread = await invoke<Item[]>('get_items', { stateFilter: 'unread' });
```

### get_item

Get a single item by ID.

**Request**:
```typescript
invoke('get_item', { id: number })
```

**Parameters**:
- `id`: Item ID

**Response**:
```typescript
Item
```

**Example**:
```typescript
const item = await invoke<Item>('get_item', { id: 123 });
```

### update_item_state

Update an item's state.

**Request**:
```typescript
invoke('update_item_state', { id: number, state: string })
```

**Parameters**:
- `id`: Item ID
- `state`: `'unread'` | `'read'` | `'archived'`

**Response**:
```typescript
void
```

**Example**:
```typescript
await invoke('update_item_state', { id: 123, state: 'read' });
```

## Source Commands

### get_sources

Get all configured sources.

**Request**:
```typescript
invoke('get_sources')
```

**Response**:
```typescript
Source[]
```

**Example**:
```typescript
const sources = await invoke<Source[]>('get_sources');
```

### add_source

Add a new source.

**Request**:
```typescript
invoke('add_source', { source: SourceInput })
```

**Parameters**:
```typescript
interface SourceInput {
  source_type: 'rss' | 'github';
  name: string;
  config_json: Record<string, any>;
  token?: string; // For GitHub sources
}
```

**Response**:
```typescript
number // Source ID
```

**Example**:
```typescript
// RSS source
const rssId = await invoke<number>('add_source', {
  source: {
    source_type: 'rss',
    name: 'Hacker News',
    config_json: {
      url: 'https://news.ycombinator.com/rss',
      poll_interval: '10m'
    }
  }
});

// GitHub source
const githubId = await invoke<number>('add_source', {
  source: {
    source_type: 'github',
    name: 'My Project',
    config_json: {
      owner: 'user',
      repo: 'repo',
      assigned_only: true
    },
    token: 'ghp_...'
  }
});
```

### update_source

Update an existing source.

**Request**:
```typescript
invoke('update_source', { id: number, update: UpdateSourceInput })
```

**Parameters**:
```typescript
interface UpdateSourceInput {
  name?: string;
  config_json?: Record<string, any>;
  enabled?: boolean;
  token?: string;
}
```

**Response**:
```typescript
void
```

**Example**:
```typescript
await invoke('update_source', {
  id: 123,
  update: {
    enabled: false
  }
});
```

### remove_source

Remove a source.

**Request**:
```typescript
invoke('remove_source', { id: number })
```

**Parameters**:
- `id`: Source ID

**Response**:
```typescript
void
```

**Example**:
```typescript
await invoke('remove_source', { id: 123 });
```

### sync_source

Manually sync a source.

**Request**:
```typescript
invoke('sync_source', { id: number })
```

**Parameters**:
- `id`: Source ID

**Response**:
```typescript
void
```

**Example**:
```typescript
await invoke('sync_source', { id: 123 });
```

## Config Commands

### get_config

Get current configuration.

**Request**:
```typescript
invoke('get_config')
```

**Response**:
```typescript
Config
```

**Example**:
```typescript
const config = await invoke<Config>('get_config');
```

### update_config

Update configuration.

**Request**:
```typescript
invoke('update_config', { config: Config })
```

**Parameters**:
```typescript
interface Config {
  github: {
    poll_interval: string;
    repos: Array<{
      owner: string;
      repo: string;
      assigned_only: boolean;
    }>;
  };
  rss: Array<{
    name: string;
    url: string;
    poll_interval: string;
  }>;
}
```

**Response**:
```typescript
void
```

**Example**:
```typescript
await invoke('update_config', {
  config: {
    github: {
      poll_interval: '5m',
      repos: []
    },
    rss: [
      {
        name: 'Hacker News',
        url: 'https://news.ycombinator.com/rss',
        poll_interval: '10m'
      }
    ]
  }
});
```

## Type Definitions

### Item

```typescript
interface Item {
  id: number;
  source_id: number;
  external_id: string;
  title: string;
  summary: string | null;
  url: string;
  item_type: 'post' | 'issue' | 'pr';
  state: 'unread' | 'read' | 'archived';
  created_at: number;
  updated_at: number;
}
```

### Source

```typescript
interface Source {
  id: number;
  type: 'rss' | 'github';
  name: string;
  config_json: Record<string, any>;
  enabled: boolean;
  last_synced_at: number | null;
}
```

## Error Handling

All commands return `Result<T, String>`:
- **Success**: Returns data
- **Error**: Returns error message string

**Example**:
```typescript
try {
  const items = await invoke<Item[]>('get_items');
} catch (error) {
  console.error('Failed to get items:', error);
}
```

## Notes

- All commands are async
- Errors are string messages (not structured)
- Database operations are atomic
- Token storage is secure (not in config)

