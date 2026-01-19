# Content Extraction System

How UmbraRelay automatically detects partial feeds and fetches full article content.

## Overview

Many RSS and Atom feeds only provide summaries or links instead of full article content. UmbraRelay automatically detects these "partial" feeds and can fetch the complete article content from the original website using article extraction algorithms.

## Architecture

### Components

1. **Content Detection** (`src-tauri/src/ingestion/content_detection.rs`)
   - Analyzes feed entries to determine if content is full or partial
   - Runs during feed ingestion/normalization

2. **Full-Text Extraction** (`src-tauri/src/ingestion/extraction.rs`)
   - Fetches article URLs and extracts readable content
   - Uses readability algorithm to extract main content
   - Sanitizes HTML for safe rendering

3. **Background Extraction Task** (`src-tauri/src/lib.rs`)
   - Asynchronous task that runs after feed sync
   - Processes items marked as "partial"
   - Updates database with extracted content

4. **Content Resolution** (`src/composables/useContent.ts`)
   - Frontend logic to determine which content to display
   - Respects user preferences
   - Handles extraction status (fetching, extracted, failed)

## Content Detection

### Detection Signals

The detection algorithm uses multiple signals to classify content:

1. **Content Length**: Character count of `content_html`
2. **HTML Structure**: Presence and count of HTML tags
3. **Content vs Summary Ratio**: Comparison of content_html length to summary length
4. **CDATA Detection**: Identifies CDATA-only content (common in Hacker News feeds)
5. **Link Density**: Detects content that's mostly links
6. **URL Availability**: Checks if article URL exists (from RSS `<link>` or Atom `<link rel="alternate">`)

### Classification Results

Items are classified as:

- **Full**: Feed provides complete article content
- **Partial**: Feed only provides summary/link, should fetch full article
- **Unknown**: Unable to determine with confidence

### Detection Logic

```rust
// High confidence: Full content
if content_length > 500 && has_html_tags && html_tag_count > 5 {
    return Full;
}

// High confidence: Partial (CDATA, mostly links, or minimal content)
if is_cdata_only || summary_is_cdata || is_mostly_link {
    return Partial;
}

// High confidence: Partial if URL exists but content is minimal
if has_url && content_length < 150 && (content_length == 0 || is_cdata_only) {
    return Partial;
}
```

### Examples

**Hacker News Feed**:
- Description contains `<![CDATA[...]]>` with just a link
- Detected as: **Partial**
- Action: Fetch full article from URL in `<link>` tag

**Full Content Feed** (e.g., some news sites):
- `content_html` has 2000+ characters with substantial HTML
- Detected as: **Full**
- Action: Use feed content directly

**Summary-Only Feed**:
- Only `summary` exists, no `content_html`
- Detected as: **Partial**
- Action: Fetch full article from URL

## Full-Text Extraction

### Process

1. **Fetch HTML**: HTTP GET request to article URL (from RSS `<link>` or Atom `<link rel="alternate">`)
2. **Extract Content**: Use readability algorithm to identify main article content
3. **Sanitize HTML**: Remove dangerous elements (scripts, iframes, etc.)
4. **Store Result**: Save extracted content and update status

### Implementation Details

- **Library**: `readabilityrs` (Rust port of Mozilla Readability)
- **Sanitization**: `ammonia` crate for HTML sanitization
- **Timeout**: 30 seconds per article
- **User-Agent**: "UmbraRelay/1.0"

### URL Source

**Important**: Extraction always uses `item.url`, which comes from:
- **RSS**: `<link>` tag in feed item
- **Atom**: `<link rel="alternate">` tag in feed entry

We do **not** parse URLs from CDATA or content_html. This ensures we fetch the canonical article URL.

### Error Handling

- **Network Errors**: Marked as "failed" with error reason
- **Parse Errors**: Marked as "failed" if readability can't extract content
- **Timeout**: 30-second timeout, marked as "failed"
- **Graceful Fallback**: Always falls back to feed content if extraction fails

## Background Extraction Task

### Execution Flow

1. **Trigger**: Runs after `normalize_and_dedupe` completes
2. **Preference Check**: Verifies user preferences (extraction enabled, view mode)
3. **Item Processing**: For each item:
   - Check if `content_completeness == "partial"`
   - Check if `content_status` is NULL or "feed_only" (not already extracted)
   - Verify URL exists
4. **Status Update**: Set `content_status = "fetching"`
5. **Extraction**: Fetch and extract article content
6. **Result Storage**: Update database with extracted content or error

### Status Lifecycle

```
feed_only (or NULL) → fetching → extracted
                              ↓
                            failed
```

- **feed_only**: Initial state, content from feed only
- **fetching**: Extraction in progress
- **extracted**: Full content successfully extracted
- **failed**: Extraction failed, will use feed content

### Rate Limiting

- **Delay**: 500ms between extractions to avoid overwhelming servers
- **Concurrent**: Extractions run sequentially (one at a time)
- **Non-Blocking**: Extraction doesn't block feed syncing

## Database Schema

### Items Table Columns

- `content_status` TEXT: Current extraction status
- `extracted_content_html` TEXT: Full extracted article content
- `content_completeness` TEXT: Detection result ("full", "partial", "unknown")
- `extraction_attempted_at` INTEGER: Timestamp when extraction was attempted
- `extraction_failed_reason` TEXT: Error message if extraction failed

### Indexes

- `idx_items_content_status`: Index on `content_status` for efficient queries

## User Preferences

### Preference Keys

- `article_view_mode`: `"auto"`, `"feed_only"`, or `"always_fetch"`
- `extraction_enabled`: `"true"` or `"false"`

### Default Values

- `article_view_mode`: `"auto"`
- `extraction_enabled`: `"true"`

### Preference Effects

**Auto Mode**:
- Detects partial feeds automatically
- Fetches full content for partial items
- Uses feed content for full items

**Feed Only Mode**:
- Never fetches full articles
- Always uses feed content

**Always Fetch Mode**:
- Attempts to fetch all articles
- Prefers extracted content, falls back to feed content

## Frontend Integration

### Content Resolution

The `useContent` composable resolves which content to display:

```typescript
const getDisplayContent = (item: Item): ContentResolution => {
  // Returns: { content, source: 'feed' | 'extracted', isFetching, hasError }
}
```

### UI Indicators

- **Spinner (⏳)**: Shown when `content_status === "fetching"`
- **"Full Article" badge**: Shown when `content_status === "extracted"`
- **"From Feed" badge**: Shown when using feed content
- **Error message**: Shown if extraction failed

### Polling Mechanism

When an item is in "fetching" status, the frontend polls every 2 seconds to check for completion. This provides real-time feedback to users.

## Performance Considerations

### Caching

- **Permanent Cache**: Once extracted, content is cached permanently
- **No Re-fetching**: Each article URL is only fetched once
- **Fast Loading**: Cached content loads instantly

### Background Processing

- **Non-Blocking**: Extraction doesn't slow down feed syncing
- **Asynchronous**: Runs in background task
- **User-Controlled**: Can be disabled via preferences

### Resource Usage

- **Bandwidth**: Only fetches when needed (partial feeds)
- **CPU**: Extraction is CPU-intensive but runs in background
- **Storage**: Extracted content stored in database (can be large)

## Error Scenarios

### Common Failures

1. **Network Timeout**: Article server is slow or unresponsive
2. **403 Forbidden**: Website blocks automated fetching
3. **404 Not Found**: Article URL is invalid or removed
4. **Parse Failure**: Readability can't identify main content
5. **Invalid HTML**: Malformed HTML that can't be parsed

### Handling

- All errors are logged with reason
- Status set to "failed" with error message
- UI falls back to feed content gracefully
- User can manually retry extraction if needed

## GitHub Token Refresh

### Proactive Refresh

On app startup, UmbraRelay proactively refreshes GitHub OAuth tokens for sources that have refresh tokens. This prevents 401 errors that would occur during sync.

### Refresh Logic

- Checks all GitHub sources (`github` and `github_notifications`)
- Identifies sources with refresh tokens
- Attempts refresh before syncing
- Logs success/failure for debugging

### Error Handling

- If refresh fails, sync will still attempt with current token
- On 401 during sync, refresh is attempted automatically
- After 3 consecutive failures, source is disabled

## Future Enhancements

- **Batch Extraction**: Extract multiple articles in parallel
- **Retry Logic**: Automatic retry for failed extractions
- **Extraction Quality**: Score extraction quality and prefer better results
- **Per-Source Override**: Allow per-source extraction preferences
- **Extraction History**: Track extraction attempts and success rates
