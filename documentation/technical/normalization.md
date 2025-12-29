# Normalization

How deduplication and normalization works in UmbraRelay.

## Overview

The normalization system ensures that:
- Duplicate items are not created
- Items are updated when external sources change
- Operations are idempotent

## Deduplication Strategy

### External ID

Each item has an `external_id` that is unique per source:

- **RSS**: Feed URL + item GUID (or link as fallback)
- **GitHub**: `{owner}/{repo}#{number}`

### Unique Constraint

Database enforces uniqueness:
```sql
UNIQUE(source_id, external_id)
```

This prevents duplicate items for the same external object.

## Upsert Logic

### Process

1. **Check Existence**: Query database for `(source_id, external_id)`
2. **If Exists**: UPDATE existing item
   - Update: `title`, `summary`, `url`, `updated_at`
   - Preserve: `state`, `created_at`
3. **If Not Exists**: INSERT new item
   - Set: All fields, `state = 'unread'`

### Idempotency

Operations are idempotent:
- Running sync multiple times produces same result
- No duplicate items created
- Existing items updated if changed

## Update Behavior

### What Updates

- **Title**: Updated if changed externally
- **Summary**: Updated if changed externally
- **URL**: Updated if changed externally
- **Updated At**: Always updated on sync

### What Preserves

- **State**: Read/unread/archived state preserved
- **Created At**: Original creation time preserved
- **ID**: Internal ID never changes

## Event Tracking

### Events Table

Optional audit trail of item updates:

- **Event Type**: `update`, `create`, etc.
- **Payload**: JSON with additional data
- **Occurred At**: Timestamp of event

### Current Usage

Events are created when:
- Item is updated (with `occurred_at` from external source)

## Example Flow

### First Sync

1. RSS feed has item with GUID `abc123`
2. Normalization checks: No existing item with `external_id = "abc123"`
3. Creates new item with `state = 'unread'`

### Subsequent Sync

1. RSS feed has same item (GUID `abc123`) with updated title
2. Normalization checks: Item exists with `external_id = "abc123"`
3. Updates existing item:
   - Updates `title` and `updated_at`
   - Preserves `state = 'read'` (if user marked as read)

### Duplicate Prevention

1. Two sources have same RSS feed URL
2. Each source has separate `source_id`
3. Items are stored separately (different `source_id`)
4. No duplicates because `(source_id, external_id)` is unique

## Edge Cases

### Missing External ID

- **RSS**: Falls back to link URL
- **GitHub**: Always has `owner/repo#number`

### External ID Changes

- **Rare**: External ID should be stable
- **If Changes**: Creates new item (old item remains)

### Concurrent Updates

- **Database**: SQLite handles concurrent writes
- **Locking**: Mutex protects database access

## Performance

- **Indexes**: `(source_id, external_id)` indexed for fast lookups
- **Queries**: Single query to check existence
- **Updates**: Efficient UPDATE vs INSERT

## Future Enhancements

- **Fuzzy Matching**: Detect similar items
- **Merge Logic**: Merge items from different sources
- **Conflict Resolution**: Handle conflicting updates

