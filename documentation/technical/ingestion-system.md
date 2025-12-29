# Ingestion System

How RSS and GitHub ingestion works in UmbraRelay.

## Overview

The ingestion system fetches items from external sources (RSS feeds and GitHub) and normalizes them into a unified format.

## Architecture

### IngestSource Trait

All ingestion sources implement the `IngestSource` trait:

```rust
pub trait IngestSource: Send + Sync {
    fn poll(&self) -> Result<Vec<IngestedItem>>;
    fn source_type(&self) -> &str;
}
```

### IngestedItem

Items are normalized into a common format:

```rust
pub struct IngestedItem {
    pub external_id: String,      // Unique ID per source
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub item_type: String,         // "post", "issue", "pr"
    pub occurred_at: Option<i64>, // Unix timestamp
}
```

## RSS Ingestion

### Implementation

- **Crate**: `rss` (RSS/Atom parser)
- **HTTP Client**: `reqwest`
- **Timeout**: 30 seconds

### Process

1. **Fetch Feed**: HTTP GET request to RSS URL
2. **Parse XML**: Parse RSS/Atom XML
3. **Extract Items**: For each item:
   - **external_id**: Item GUID or link (fallback)
   - **title**: Item title
   - **summary**: Item description
   - **url**: Item link
   - **item_type**: Always "post"
   - **occurred_at**: Parsed pub_date (if available)

### Error Handling

- **Network Errors**: Retry logic (future)
- **Parse Errors**: Log and skip invalid items
- **Timeout**: 30-second timeout

## GitHub Ingestion

### Implementation

- **API**: GitHub REST API v3
- **HTTP Client**: `reqwest`
- **Authentication**: Personal Access Token
- **Timeout**: 30 seconds

### Process

1. **Fetch Issues**: GET `/repos/{owner}/{repo}/issues`
2. **Query Parameters**:
   - `state=open`: Only open issues/PRs
   - `filter=assigned`: Only assigned (if `assigned_only`)
   - `per_page=100`: Maximum items per request
3. **Parse Response**: JSON response parsed into `GitHubIssue`
4. **Extract Items**: For each issue:
   - **external_id**: `{owner}/{repo}#{number}`
   - **title**: Issue/PR title
   - **summary**: Issue/PR body (truncated to 500 chars)
   - **url**: `html_url`
   - **item_type**: "pr" if has `pull_request`, else "issue"
   - **occurred_at**: Parsed `updated_at` timestamp

### Authentication

- **Token Storage**: Secure storage (Tauri state)
- **Header**: `Authorization: token {token}`
- **User-Agent**: "UmbraRelay"

### Error Handling

- **401 Unauthorized**: Invalid or expired token
- **403 Forbidden**: Rate limit or permission issue
- **404 Not Found**: Repository doesn't exist
- **Rate Limits**: 5000 requests/hour (authenticated)

## Polling Intervals

### Configuration

- **RSS**: Per-source `poll_interval` in config
- **GitHub**: Global `poll_interval` in config
- **Format**: `5m`, `10m`, `1h`, etc.
- **Default**: 10 minutes (RSS), 5 minutes (GitHub)

### Background Service

- **Service**: Tokio async task
- **Check Frequency**: Every 60 seconds
- **Sync Logic**: Compares `last_synced_at` with poll interval
- **Concurrent**: Sources polled independently

## Rate Limiting

### GitHub

- **Authenticated**: 5000 requests/hour
- **Per-Request**: 1 request per sync
- **Recommendation**: Poll interval >= 5 minutes

### RSS

- **No Limits**: But be respectful
- **Recommendation**: Poll interval >= 10 minutes

## Error Recovery

- **Network Errors**: Logged, retry on next cycle
- **Parse Errors**: Logged, skip invalid items
- **Auth Errors**: Logged, source disabled (future)

## Future Enhancements

- **Pagination**: Handle large result sets
- **Webhooks**: Real-time updates (GitHub)
- **Retry Logic**: Exponential backoff
- **Caching**: ETag/Last-Modified headers
- **Parallel Polling**: Poll multiple sources concurrently

