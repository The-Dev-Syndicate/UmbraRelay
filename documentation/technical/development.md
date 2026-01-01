# Development Guide

Setup and contribution guide for UmbraRelay.

## Prerequisites

- **Rust**: Latest stable (1.70+)
- **Node.js**: v18 or later
- **npm**: Comes with Node.js
- **Git**: For version control

## Development Setup

### 1. Clone Repository

```bash
git clone https://github.com/dev-syndicate/UmbraRelay.git
cd UmbraRelay
```

### 2. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Rust dependencies are managed by Cargo (automatic)
```

### 3. Run in Development Mode

```bash
npm run tauri dev
```

This will:
- Start Vite dev server (frontend)
- Compile Rust backend
- Launch app with hot-reload

## Project Structure

```
UmbraRelay/
├── src/                    # Vue frontend
│   ├── components/        # Vue components
│   ├── composables/        # Vue composables
│   └── types.ts           # TypeScript types
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── storage/       # Database layer
│   │   ├── config/        # Configuration
│   │   ├── ingestion/     # RSS/GitHub ingestion
│   │   ├── normalization/ # Deduplication
│   │   └── commands.rs    # Tauri commands
│   └── Cargo.toml         # Rust dependencies
├── documentation/          # Documentation
└── package.json           # Node.js dependencies
```

## Building

### Development Build

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Outputs:
- **macOS**: `.app` bundle
- **Windows**: `.msi` installer
- **Linux**: `.AppImage`

## Testing

### Manual Testing

1. **RSS Feed**: Add a test RSS feed
2. **GitHub**: Add a test GitHub source (with token)
3. **Sync**: Test manual sync
4. **Items**: Test viewing and updating items

### Automated Testing

(Coming soon)

## Code Style

### Rust

- Follow Rust style guide
- Use `rustfmt` for formatting
- Use `clippy` for linting

```bash
cargo fmt
cargo clippy
```

### TypeScript/Vue

- Follow Vue 3 style guide
- Use ESLint (if configured)
- Use Prettier (if configured)

## Contributing

### Workflow

1. **Fork** the repository
2. **Create** a feature branch
3. **Make** your changes
4. **Test** thoroughly
5. **Submit** a pull request

### Pull Request Guidelines

- **Description**: Clear description of changes
- **Testing**: Describe how you tested
- **Documentation**: Update docs if needed
- **Breaking Changes**: Note any breaking changes

## Architecture Decisions

### Why Tauri?

- **Performance**: Native performance
- **Size**: Smaller bundle than Electron
- **Security**: Better security model
- **Rust**: Type-safe backend

### Why Vue 3?

- **Reactivity**: Excellent reactivity system
- **Composition API**: Better code organization
- **TypeScript**: Type safety
- **Ecosystem**: Rich ecosystem

### Why SQLite?

- **Local-first**: No external database needed
- **Embedded**: No separate server
- **Reliable**: Battle-tested
- **Portable**: Single file database

## Debugging

### Frontend

- **DevTools**: Use browser dev tools
- **Console**: Check console for errors
- **Vue DevTools**: Install Vue DevTools extension

### Backend

- **Logs**: Check console output
- **Rust Debugger**: Use `rust-gdb` or `lldb`
- **Tauri Logs**: Check Tauri console

### Database

```bash
# Inspect database
sqlite3 ~/Library/Application\ Support/com.devsyndicate.umbra-relay/umbrarelay.db

# Run queries
sqlite3 umbrarelay.db "SELECT * FROM items LIMIT 10;"
```

## Common Issues

### Build Errors

- **Rust**: Update Rust toolchain
- **Node**: Update Node.js
- **Dependencies**: Run `npm install` again

### Runtime Errors

- **Database**: Check database file permissions
- **Config**: Check source configuration in database
- **Tokens**: Check token storage

## Future Work

- **Testing**: Unit and integration tests
- **CI/CD**: Automated builds and tests
- **Packaging**: Better distribution
- **Plugins**: Plugin system
- **Notifications**: Desktop notifications

## Resources

- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)

## Getting Help

- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions
- **Documentation**: Check other docs

