# Architecture

High-level architecture overview of UmbraRelay.

## System Overview

UmbraRelay is a desktop application built with:
- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Database**: SQLite (embedded)
- **Communication**: Tauri commands (IPC)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Vue 3 Frontend                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  InboxView   │  │  ItemDetail  │  │ SourceConfig │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                        ↕ Tauri Commands
┌─────────────────────────────────────────────────────────┐
│                  Rust Backend (Tauri)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Config     │  │  Ingestion   │  │   Storage    │  │
│  │  (TOML + UI) │  │  (RSS/GitHub)│  │   (SQLite)   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Background Polling Service                │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Component Overview

### Frontend Components

- **InboxView**: Displays list of items with filtering
- **ItemDetail**: Shows individual item details
- **SourceConfig**: Manages RSS and GitHub sources

### Backend Modules

- **storage/**: Database operations and models
- **config/**: TOML parsing and secure token storage
- **ingestion/**: RSS and GitHub API integration
- **normalization/**: Deduplication and upsert logic
- **commands/**: Tauri command handlers

## Data Flow

### Adding a Source

1. User fills form in `SourceConfig.vue`
2. Frontend calls `add_source` Tauri command
3. Backend stores source in database
4. Token stored securely (if GitHub)
5. Source appears in UI

### Polling a Source

1. Background service checks poll intervals
2. Creates appropriate ingester (RSS/GitHub)
3. Fetches items from external source
4. Normalizes and deduplicates items
5. Stores/updates items in database
6. Updates `last_synced_at` timestamp

### Viewing Items

1. User navigates to Inbox
2. Frontend calls `get_items` Tauri command
3. Backend queries database
4. Returns items to frontend
5. Frontend displays items

### Updating Item State

1. User clicks action (Mark Read, Archive, etc.)
2. Frontend calls `update_item_state` Tauri command
3. Backend updates database
4. Frontend updates UI

## Background Polling Service

The background polling service:
- Runs in a Tokio async task
- Checks enabled sources periodically
- Respects poll intervals per source
- Handles errors gracefully
- Updates sync timestamps

## Security Considerations

- **GitHub Tokens**: Stored securely using Tauri's secure storage
- **Local Data**: All data stored locally (no cloud sync)
- **No Network**: App only makes outbound requests (no server)
- **Permissions**: Minimal system permissions required

## Database

- **Location**: App data directory
- **File**: `umbrarelay.db`
- **Type**: SQLite (embedded)
- **Migrations**: Versioned migrations on startup

## Configuration

- **Format**: TOML
- **Location**: App data directory
- **Name**: `umbrarelay.toml`
- **Reload**: Requires app restart (hot-reload coming)

## Error Handling

- **Frontend**: Try-catch with user-friendly messages
- **Backend**: Result types with error propagation
- **Logging**: Errors logged to console (structured logging coming)

## Performance

- **Database**: Indexed queries for fast retrieval
- **Polling**: Async background service (non-blocking)
- **UI**: Reactive Vue components (efficient updates)
- **Caching**: Items cached in memory (Vue reactivity)

## Future Enhancements

- Hot-reload configuration
- Structured logging
- Bulk operations
- Keyboard shortcuts
- Notifications
- Plugin system

