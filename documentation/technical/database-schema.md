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
| type | TEXT | NOT NULL | Source type: `rss`, `atom`, `github`, or `github_notifications` |
| name | TEXT | NOT NULL | Friendly name for the source |
| config_json | TEXT | NOT NULL | JSON configuration (source-specific) |
| enabled | INTEGER | NOT NULL DEFAULT 1 | 1 = enabled, 0 = disabled |
| last_synced_at | INTEGER | NULL | Unix timestamp of last sync |
| secret_id | INTEGER | NULL, FK → secrets(id) | Reference to secret for authentication |
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
| item_type | TEXT | NOT NULL | Type: `rss`, `atom`, `issue`, `pr`, etc. |
| state | TEXT | NOT NULL DEFAULT 'unread' | State: `unread`, `read`, `archived` |
| image_url | TEXT | NULL | Optional image URL for the item |
| content_html | TEXT | NULL | Optional HTML content |
| author | TEXT | NULL | Item author |
| category | TEXT | NULL | JSON array of categories |
| comments | TEXT | NULL | Comments URL or count |
| thread_id | TEXT | NULL | Thread ID for grouping related items |
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

## Additional Tables

### groups

Stores source groups for organization.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| name | TEXT | NOT NULL UNIQUE | Group name |
| created_at | INTEGER | NOT NULL | Unix timestamp of creation |
| updated_at | INTEGER | NOT NULL | Unix timestamp of last update |

### source_groups

Junction table linking sources to groups (many-to-many relationship).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| source_id | INTEGER | NOT NULL, FK → sources(id) | Reference to source |
| group_id | INTEGER | NOT NULL, FK → groups(id) | Reference to group |
| PRIMARY KEY | (source_id, group_id) | | Composite primary key |

### custom_views

Stores user-defined custom views for filtering items.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| name | TEXT | NOT NULL | View name |
| source_ids | TEXT | NULL | JSON array of source IDs |
| group_names | TEXT | NULL | JSON array of group names |
| created_at | INTEGER | NOT NULL | Unix timestamp of creation |
| updated_at | INTEGER | NOT NULL | Unix timestamp of last update |

### user_preferences

Stores user preferences as key-value pairs.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| key | TEXT | PRIMARY KEY | Preference key |
| value | TEXT | NOT NULL | Preference value (JSON) |

### secrets

Stores secret metadata (actual values stored securely elsewhere).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-incrementing ID |
| name | TEXT | NOT NULL UNIQUE | Secret name |
| ttl_type | TEXT | NOT NULL DEFAULT 'forever' | TTL type: `forever`, `relative`, `absolute` |
| ttl_value | TEXT | NULL | TTL value (format depends on type) |
| expires_at | INTEGER | NULL | Unix timestamp of expiration |
| refresh_token_id | INTEGER | NULL | Reference to refresh token secret |
| refresh_failure_count | INTEGER | NOT NULL DEFAULT 0 | Count of refresh failures |
| created_at | INTEGER | NOT NULL | Unix timestamp of creation |
| updated_at | INTEGER | NOT NULL | Unix timestamp of last update |

## Indexes

- `idx_items_source_id` on `items(source_id)` - Fast source queries
- `idx_items_state` on `items(state)` - Fast state filtering
- `idx_items_created_at` on `items(created_at)` - Fast date sorting
- `idx_items_thread_id` on `items(thread_id)` - Fast thread grouping
- `idx_events_item_id` on `events(item_id)` - Fast event queries
- `idx_groups_name` on `groups(name)` - Fast group lookups
- `idx_source_groups_source_id` on `source_groups(source_id)` - Fast source-group queries
- `idx_source_groups_group_id` on `source_groups(group_id)` - Fast group-source queries
- `idx_custom_views_name` on `custom_views(name)` - Fast view lookups
- `idx_user_preferences_key` on `user_preferences(key)` - Fast preference lookups
- `idx_secrets_name` on `secrets(name)` - Fast secret lookups
- `idx_secrets_expires_at` on `secrets(expires_at)` - Fast expiration queries
- `idx_sources_secret_id` on `sources(secret_id)` - Fast secret-source queries

## Relationships

```
sources (1) ──→ (many) items
sources (many) ←→ (many) groups (via source_groups)
items (1) ──→ (many) events
sources (many) ──→ (1) secrets
```

- **Cascade Deletes**: Deleting a source deletes all its items and source_group relationships
- **Cascade Deletes**: Deleting an item deletes all its events
- **Cascade Deletes**: Deleting a group deletes all source_group relationships

## Data Integrity

- **Foreign Keys**: Enforced by SQLite
- **Unique Constraints**: Prevent duplicate items per source
- **NOT NULL**: Required fields enforced
- **Defaults**: Sensible defaults for optional fields

## Migrations

Migrations run automatically on startup using the `rusqlite_migration` crate.

### Current Migration Strategy (v1.0+)

As of v1.0, we use a single consolidated migration that represents the final schema state. This simplifies new installations while maintaining compatibility with existing databases through the migration system.

The consolidated migration includes:
- All tables (sources, items, events, groups, source_groups, custom_views, user_preferences, secrets)
- All indexes
- All foreign key relationships
- Complete schema with all columns

### Legacy Migration Support

Existing databases that were created with previous versions will automatically migrate to the latest schema. The migration system handles this transparently.

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

