# Windows Debugging Guide

If UmbraRelay flashes open and immediately closes on Windows, follow these steps to diagnose the issue.

## Step 1: Check Error Log

The app automatically logs errors to a file. Check these locations:

1. **AppData Location** (recommended):
   - `%APPDATA%\UmbraRelay\umbrarelay_error.log`
   - Or: `%LOCALAPPDATA%\UmbraRelay\umbrarelay_error.log`

2. **Current Directory** (fallback):
   - `umbrarelay_error.log` (in the directory where you ran the app)

Open the log file to see what error occurred during startup.

## Step 2: Enable Console Window (for debugging)

To see error messages in a console window, you have two options:

### Option A: Build in Debug Mode

Debug builds automatically show the console window:

```bash
# Build in debug mode
npm run tauri build -- --debug
```

### Option B: Temporarily Enable Console in Release Build

Edit `src-tauri/src/main.rs` and comment out the console suppression line:

```rust
// Comment out this line to enable console window:
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

Then rebuild:

```bash
npm run tauri build
```

## Step 3: Common Issues and Solutions

### Issue: "Failed to get app data directory"

**Cause**: Windows permissions issue or path problem.

**Solution**:
- Run the app as Administrator (right-click → Run as Administrator)
- Check if the AppData directory exists and is accessible
- Check Windows Event Viewer for permission errors

### Issue: "Failed to create app data directory"

**Cause**: Insufficient permissions to create directories.

**Solution**:
- Run as Administrator
- Check disk space
- Verify write permissions in AppData folder

### Issue: "Failed to initialize database"

**Cause**: Database file corruption or permission issues.

**Solution**:
- Delete the database file: `%APPDATA%\UmbraRelay\umbrarelay.db`
- Run as Administrator
- Check if another instance of the app is running

### Issue: "Invalid database path"

**Cause**: Path contains non-UTF8 characters (rare on Windows).

**Solution**:
- Move the app to a path without special characters
- Use a standard Windows username (avoid special characters)

## Step 4: Manual Testing

If the error log doesn't provide enough information, try running the app from command line:

1. Open Command Prompt or PowerShell
2. Navigate to the app directory
3. Run the executable directly:
   ```cmd
   cd "C:\path\to\UmbraRelay"
   .\umbra-relay_0.0.1_x64-setup.exe
   ```

This may show error messages in the console.

## Step 5: Check Windows Event Viewer

Windows Event Viewer may have additional error information:

1. Press `Win + R`
2. Type `eventvwr.msc` and press Enter
3. Navigate to: **Windows Logs** → **Application**
4. Look for errors related to UmbraRelay

## Reporting Issues

When reporting Windows issues, please include:

1. Contents of `umbrarelay_error.log`
2. Windows version (e.g., Windows 10, Windows 11)
3. Whether running as Administrator helps
4. Any relevant entries from Windows Event Viewer
5. The exact error message (if visible)

## Quick Fix: Clean Install

If all else fails, try a clean install:

1. Uninstall UmbraRelay
2. Delete the app data directory: `%APPDATA%\UmbraRelay` or `%LOCALAPPDATA%\UmbraRelay`
3. Reinstall the app
4. Run as Administrator on first launch

