# Database Schema

Complete documentation of the UmbraRelay SQLite database schema.

## Database File

- **Location**: App data directory
- **Filename**: `umbrarelay.db`
- **Type**: SQLite 3
- **Migrations**: Automatic on startup

## Tables

### sources

Stores configured RSS feeds and GitHub sources.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| type | TEXT | NOT NULL | Source type: `rss` or `github` |
| name | TEXT | NOT NULL | Friendly name for the source |
| config_json | TEXT | NOT NULL | JSON configuration (source-specific) |
| enabled | INTEGER | NOT NULL DEFAULT 1 | 1 = enabled, 0 = disabled |
| last_synced_at | INTEGER | NULL | Unix timestamp of last sync |
| created_at | INTEGER | NOT NULL | Unix timestamp of creation |
| updated_at | INTEGER | NOT NULL | Unix timestamp of last update |

**Example config_json**:
- RSS: `{"url": "https://example.com/feed.xml", "poll_interval": "10m"}`
- GitHub: `{"owner": "user", "repo": "repo", "assigned_only": true}`

### items

Stores inbox items from all sources.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| source_id | INTEGER | NOT NULL, FK → sources(id) | Reference to source |
| external_id | TEXT | NOT NULL | External ID (unique per source) |
| title | TEXT | NOT NULL | Item title |
| summary | TEXT | NULL | Item summary/description |
| url | TEXT | NOT NULL | External URL |
| item_type | TEXT | NOT NULL | Type: `post`, `issue`, `pr` |
| state | TEXT | NOT NULL DEFAULT 'unread' | State: `unread`, `read`, `archived` |
| created_at | INTEGER | NOT NULL | Unix timestamp of creation |
| updated_at | INTEGER | NOT NULL | Unix timestamp of last update |

**Unique Constraint**: `(source_id, external_id)` - Prevents duplicates

**External ID Examples**:
- RSS: Feed URL + item GUID
- GitHub: `owner/repo#number`

### events

Optional audit trail of item updates.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| item_id | INTEGER | NOT NULL, FK → items(id) | Reference to item |
| event_type | TEXT | NOT NULL | Event type (e.g., `update`) |
| payload_json | TEXT | NULL | JSON payload (event-specific) |
| occurred_at | INTEGER | NOT NULL | Unix timestamp of event |

## Indexes

- `idx_items_source_id` on `items(source_id)` - Fast source queries
- `idx_items_state` on `items(state)` - Fast state filtering
- `idx_items_created_at` on `items(created_at)` - Fast date sorting
- `idx_events_item_id` on `events(item_id)` - Fast event queries

## Relationships

```
sources (1) ──→ (many) items
items (1) ──→ (many) events
```

- **Cascade Deletes**: Deleting a source deletes all its items
- **Cascade Deletes**: Deleting an item deletes all its events

## Data Integrity

- **Foreign Keys**: Enforced by SQLite
- **Unique Constraints**: Prevent duplicate items per source
- **NOT NULL**: Required fields enforced
- **Defaults**: Sensible defaults for optional fields

## Migrations

Migrations are versioned and run automatically on startup:
- Version 1: Initial schema (sources, items, events)
- Version 2: Indexes for performance

Migrations use `rusqlite_migration` crate.

## Backup and Recovery

### Backup

```bash
# Copy database file
cp ~/Library/Application\ Support/com.devsyndicate.umbra-relay/umbrarelay.db ~/backup/
```

### Recovery

```bash
# Restore from backup
cp ~/backup/umbrarelay.db ~/Library/Application\ Support/com.devsyndicate.umbra-relay/
```

### Export Data

Use SQLite tools to export:
```bash
sqlite3 umbrarelay.db ".dump" > backup.sql
```

## Performance Considerations

- **Indexes**: Optimize common queries
- **Queries**: Use indexed columns in WHERE clauses
- **Pagination**: Consider for large datasets (future)
- **Vacuum**: Periodic VACUUM recommended for large databases

## Schema Evolution

Future schema changes will:
- Use migrations (versioned)
- Preserve existing data
- Add new columns as nullable initially
- Deprecate old columns gradually

