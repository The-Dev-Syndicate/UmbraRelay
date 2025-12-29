# Installation

This guide covers how to install UmbraRelay on your system.

## System Requirements

- **macOS**: 10.13 or later
- **Windows**: Windows 10 or later
- **Linux**: Most modern distributions (tested on Ubuntu 20.04+)

## Installation Methods

### Option 1: Build from Source

1. **Prerequisites**:
   - [Rust](https://www.rust-lang.org/tools/install) (latest stable)
   - [Node.js](https://nodejs.org/) (v18 or later)
   - [npm](https://www.npmjs.com/) (comes with Node.js)

2. **Clone the repository**:
   ```bash
   git clone https://github.com/dev-syndicate/UmbraRelay.git
   cd UmbraRelay
   ```

3. **Install dependencies**:
   ```bash
   npm install
   ```

4. **Build the application**:
   ```bash
   npm run tauri build
   ```

5. **Find the installer**:
   - **macOS**: `src-tauri/target/release/bundle/macos/UmbraRelay.app`
   - **Windows**: `src-tauri/target/release/bundle/msi/UmbraRelay_0.1.0_x64_en-US.msi`
   - **Linux**: `src-tauri/target/release/bundle/appimage/UmbraRelay_0.1.0_amd64.AppImage`

### Option 2: Pre-built Binaries (When Available)

Download the latest release from the [Releases page](https://github.com/dev-syndicate/UmbraRelay/releases).

- **macOS**: Download `.dmg` file and drag to Applications
- **Windows**: Download `.msi` installer and run
- **Linux**: Download `.AppImage`, make executable (`chmod +x`), and run

## Development Mode

To run UmbraRelay in development mode:

```bash
npm run tauri dev
```

This will:
- Start the Vite dev server
- Compile the Rust backend
- Launch the application with hot-reload enabled

## Verifying Installation

1. Launch UmbraRelay
2. You should see the main window with "Inbox" and "Sources" navigation
3. The app should start without errors

## Next Steps

- [First Run Guide](first-run.md) - Set up your first sources
- [Basic Usage](basic-usage.md) - Learn the interface

## Troubleshooting

If you encounter issues during installation:

- **Build errors**: Ensure Rust and Node.js are up to date
- **Permission errors**: Check file permissions on your system
- **Missing dependencies**: See [Development Guide](../technical/development.md) for detailed setup

For more help, see the [Troubleshooting Guide](../user-guide/troubleshooting.md).

