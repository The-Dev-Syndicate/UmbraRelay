# Technical Documentation

Technical documentation for developers and contributors.

## Table of Contents

1. [Architecture](architecture.md) - System architecture overview
2. [Database Schema](database-schema.md) - SQLite database structure
3. [Ingestion System](ingestion-system.md) - How RSS and GitHub ingestion works
4. [Normalization](normalization.md) - Deduplication and normalization logic
5. [API Reference](api-reference.md) - Tauri command API reference
6. [Development](development.md) - Development setup and contribution guide

## Target Audience

This documentation is for:
- **Developers**: Contributing to UmbraRelay
- **Contributors**: Understanding the codebase
- **Advanced Users**: Customizing or extending UmbraRelay

## Code Organization

UmbraRelay is organized into modules:

- **storage/**: Database models and CRUD operations
- **config/**: Configuration management (TOML, secure storage)
- **ingestion/**: RSS and GitHub ingestion
- **normalization/**: Deduplication and normalization
- **commands/**: Tauri command handlers

## Quick Links

- [Development Setup](development.md) - Get started developing
- [Architecture Overview](architecture.md) - Understand the system
- [API Reference](api-reference.md) - Frontend-backend communication

