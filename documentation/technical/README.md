# Technical Documentation

Technical documentation for developers and contributors.

## Table of Contents

1. [Architecture](architecture.md) - System architecture overview
2. [Database Schema](database-schema.md) - SQLite database structure
3. [Ingestion System](ingestion-system.md) - How RSS and GitHub ingestion works
4. [Content Extraction](content-extraction.md) - Automatic full-text article extraction
5. [Normalization](normalization.md) - Deduplication and normalization logic
6. [API Reference](api-reference.md) - Tauri command API reference
7. [Console Commands](console-commands.md) - Power user console commands
8. [Development](development.md) - Development setup and contribution guide
9. [Release Process](release-process.md) - Creating and publishing releases
10. [Auto-Updates](auto-updates.md) - How the automatic update system works

## Target Audience

This documentation is for:
- **Developers**: Contributing to UmbraRelay
- **Contributors**: Understanding the codebase
- **Advanced Users**: Customizing or extending UmbraRelay

## Code Organization

UmbraRelay is organized into modules:

- **storage/**: Database models and CRUD operations
- **config/**: Secure token storage
- **ingestion/**: RSS and GitHub ingestion, content detection, and extraction
- **normalization/**: Deduplication and normalization
- **commands/**: Tauri command handlers

## Quick Links

- [Development Setup](development.md) - Get started developing
- [Architecture Overview](architecture.md) - Understand the system
- [API Reference](api-reference.md) - Frontend-backend communication

