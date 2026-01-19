# UmbraRelay Documentation

Welcome to the UmbraRelay documentation. UmbraRelay is a local-first desktop inbox aggregator that runs entirely on your machine.

## Documentation Structure

### Getting Started
- [Installation](getting-started/installation.md) - How to install UmbraRelay
- [First Run](getting-started/first-run.md) - Setting up UmbraRelay for the first time
- [Basic Usage](getting-started/basic-usage.md) - Learn the basics of using UmbraRelay

### User Guide
- [Inbox Management](user-guide/inbox-management.md) - Managing your inbox items
- [Adding Sources](user-guide/adding-sources.md) - Adding RSS feeds and GitHub sources
- [Configuration](user-guide/configuration.md) - Configuring UmbraRelay settings
- [Troubleshooting](user-guide/troubleshooting.md) - Common issues and solutions

### Technical Documentation
- [Architecture](technical/architecture.md) - System architecture overview
- [Database Schema](technical/database-schema.md) - SQLite database structure
- [Ingestion System](technical/ingestion-system.md) - How RSS and GitHub ingestion works
- [Content Extraction](technical/content-extraction.md) - Automatic full-text article extraction
- [Normalization](technical/normalization.md) - Deduplication and normalization logic
- [API Reference](technical/api-reference.md) - Tauri command API reference
- [Development](technical/development.md) - Development setup and contribution guide
- [Release Process](technical/release-process.md) - Creating and publishing releases
- [Auto-Updates](technical/auto-updates.md) - How the automatic update system works

### Examples
- [Source Setup Examples](examples/config-examples.md) - Example source configurations
- [Use Cases](examples/use-cases.md) - Real-world usage scenarios

## Quick Links

- **New to UmbraRelay?** Start with [Installation](getting-started/installation.md)
- **Want to add sources?** See [Adding Sources](user-guide/adding-sources.md)
- **Having issues?** Check [Troubleshooting](user-guide/troubleshooting.md)
- **Want to contribute?** Read [Development Guide](technical/development.md)

## About UmbraRelay

UmbraRelay is a self-contained inbox aggregator that:
- Runs entirely on your machine (local-first)
- Aggregates RSS feeds and GitHub issues/PRs
- Provides a unified inbox interface
- Stores all data locally in SQLite
- Requires no external services or cloud dependencies

The name "UmbraRelay" comes from:
- **Umbra** = shadow/quiet (your data stays private)
- **Relay** = passing signals forward (aggregating information)

