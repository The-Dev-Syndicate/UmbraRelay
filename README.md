# UmbraRelay

A local-first desktop inbox aggregator that runs entirely on your machine.

## Overview

UmbraRelay aggregates RSS feeds and GitHub issues/PRs into a unified inbox. All data lives on your machine - no external databases, no cloud services, no tracking.

**Umbra** = shadow/quiet (your data stays private)  
**Relay** = passing signals forward (aggregating information)

## Features

- **RSS Feeds**: Aggregate multiple RSS feeds
- **GitHub Integration**: Track issues and PRs assigned to you
- **Unified Inbox**: Single interface for all sources
- **Local-First**: All data stored locally in SQLite
- **Automatic Polling**: Background sync at configurable intervals

## Quick Start

### Installation

See [Installation Guide](documentation/getting-started/installation.md) for detailed instructions.

**Development**:
```bash
npm install
npm run tauri dev
```

**Build**:
```bash
npm run tauri build
```

### First Run

1. Launch UmbraRelay
2. Navigate to "Sources"
3. Add an RSS feed or GitHub source
4. Click "Sync" to fetch items
5. View items in "Inbox"

See [First Run Guide](documentation/getting-started/first-run.md) for details.

## Documentation

Comprehensive documentation is available in the `documentation/` folder:

- **[Getting Started](documentation/getting-started/)** - Installation and basic usage
- **[User Guide](documentation/user-guide/)** - Detailed feature documentation
- **[Technical Docs](documentation/technical/)** - Architecture and development
- **[Console Commands](documentation/technical/console-commands.md)** - Power user console commands
- **[Examples](documentation/examples/)** - Configuration examples and use cases


## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Database**: SQLite (embedded)
- **Communication**: Tauri commands (IPC)

## Development

See [Development Guide](documentation/technical/development.md) for setup and contribution guidelines.

## License

Pending - All rights reserved I guess for now.

## Contributing

Contributions welcome! See [Development Guide](documentation/technical/development.md) for details.
