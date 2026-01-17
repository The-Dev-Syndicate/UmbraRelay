# (Not Implmented Yet) Auto-Update System

How UmbraRelay's automatic update system works for end users.

## Overview

UmbraRelay includes an automatic update system that:
- Checks for new versions when the app starts
- Downloads updates from GitHub releases
- Installs updates with a single click
- Verifies update integrity with cryptographic signatures
- Works seamlessly across macOS, Windows, and Linux

## User Experience

### Automatic Update Check

When a user opens UmbraRelay:

1. **App starts** → Checks for updates in the background
2. **If update found** → Shows dialog:
   ```
   ┌─────────────────────────────────────┐
   │  Update Available                   │
   ├─────────────────────────────────────┤
   │  Version 1.0.1 is now available!     │
   │                                      │
   │  [Release notes from GitHub]        │
   │                                      │
   │  [Install Update]  [Later]          │
   └─────────────────────────────────────┘
   ```

3. **User clicks "Install Update"**:
   - App downloads update in background
   - Shows progress indicator
   - Verifies signature
   - Installs update
   - Relaunches app automatically
   - User is now on the new version

4. **User clicks "Later"**:
   - Dialog closes
   - App continues normally
   - Update check happens again on next startup

### Manual Update Check

Users can also manually check for updates:
- Via menu: **Help → Check for Updates**
- Or programmatically via the update API

## Technical Flow

### High-Level Process

```
┌─────────────┐
│ App Startup │
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│ Check Update API  │ ← GET latest.json from GitHub
│ (GitHub Releases) │
└──────┬───────────┘
       │
       ├─→ Current == Latest? → No update, continue
       │
       └─→ Current < Latest? → Show update dialog
              │
              ▼
       ┌─────────────────┐
       │ Update Dialog     │
       │ "Install" | "Later"│
       └─────┬─────────────┘
             │
    ┌────────┴────────┐
    │                  │
    ▼                  ▼
┌─────────┐      ┌──────────┐
│ Install │      │  Later   │
└────┬────┘      └──────────┘
     │
     ▼
┌─────────────────┐
│ Download Update  │ ← Downloads platform binary
│ (with progress)  │   (.msi, .app, .AppImage)
└──────┬───────────┘
       │
       ▼
┌─────────────────┐
│ Verify Signature│ ← Cryptographic verification
└──────┬───────────┘
       │
       ▼
┌─────────────────┐
│ Install Update  │ ← Platform-specific installation
└──────┬───────────┘
       │
       ▼
┌─────────────────┐
│ Relaunch App    │ ← Restarts with new version
└─────────────────┘
```

### Update Manifest

The update system uses a manifest file (`latest.json`) hosted on GitHub releases:

```json
{
  "version": "1.0.1",
  "notes": "Bug fixes and improvements",
  "pub_date": "2024-01-15T10:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../UmbraRelay_1.0.1_x64.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/.../UmbraRelay_1.0.1_aarch64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../UmbraRelay_1.0.1_x64-setup.exe"
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../UmbraRelay_1.0.1_amd64.AppImage"
    }
  }
}
```

## Implementation Details

### Backend (Rust)

The updater is implemented using Tauri's built-in updater plugin:

```rust
// src-tauri/src/lib.rs
use tauri_plugin_updater::UpdaterBuilder;

pub fn run() {
    tauri::Builder::default()
        .plugin(
            UpdaterBuilder::new()
                .build()
        )
        // ... other plugins
}
```

### Frontend (Vue/TypeScript)

Update checking is handled via the Tauri updater API:

```typescript
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/api/process';

async function checkForUpdates() {
  const update = await check();
  
  if (update?.available) {
    // Show dialog
    const confirmed = await ask(
      `Update ${update.version} available!\n\n${update.body}\n\nInstall now?`
    );
    
    if (confirmed) {
      await update.downloadAndInstall();
      await relaunch();
    }
  }
}
```

### Configuration

Update endpoint is configured in `tauri.conf.json`:

```json
{
  "app": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/The-Dev-Syndicate/UmbraRelay/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

## Security

### Cryptographic Signing

All updates are cryptographically signed:
- **Private key**: Used by developers to sign releases (kept secret)
- **Public key**: Embedded in app to verify updates
- **Verification**: App verifies signature before installing

### Key Generation

Generate signing keys once:

```bash
tauri signer generate -w ~/.tauri/umbrarelay.key
```

This creates:
- Private key: `~/.tauri/umbrarelay.key` (keep secret!)
- Public key: Add to `tauri.conf.json`

## Release Workflow Integration

The GitHub Actions release workflow automatically:
1. Builds binaries for all platforms
2. Generates update manifest (`latest.json`)
3. Signs binaries with private key
4. Uploads everything to GitHub release
5. Makes manifest available at update endpoint

## Update Frequency

### Automatic Checks

- **On app startup**: Checks once per session
- **Background**: Can be configured to check periodically
- **Manual**: User can check anytime via menu

### Update Behavior

- **Silent check**: Happens in background, doesn't interrupt user
- **User choice**: User decides when to install
- **Non-blocking**: App continues normally if user chooses "Later"

## Platform-Specific Notes

### macOS
- Updates `.app` bundle
- May require user to allow app in System Preferences
- Code signing required for distribution

### Windows
- Updates `.msi` installer
- May show UAC prompt
- Requires code signing certificate for production

### Linux
- Updates `.AppImage` file
- May require making file executable
- No code signing required (but signature verification still works)

## Troubleshooting

### Update Check Fails

**Symptoms**: App can't check for updates

**Possible causes**:
- No internet connection
- GitHub API rate limiting
- Firewall blocking requests
- Update endpoint misconfigured

**Solutions**:
- Check internet connection
- Verify endpoint URL in config
- Check firewall settings
- Wait and retry (if rate limited)

### Update Download Fails

**Symptoms**: Download starts but fails

**Possible causes**:
- Network interruption
- Insufficient disk space
- File permissions issue

**Solutions**:
- Check internet connection
- Free up disk space
- Check file permissions
- Retry update

### Signature Verification Fails

**Symptoms**: Update fails with "signature invalid"

**Possible causes**:
- Corrupted download
- Wrong public key in config
- Man-in-the-middle attack (rare)

**Solutions**:
- Retry update (may be corrupted download)
- Verify public key matches private key
- Contact developers if persistent

### Update Installs But App Doesn't Relaunch

**Symptoms**: Update installs but old version still runs

**Possible causes**:
- Relaunch failed
- App locked by another process

**Solutions**:
- Manually restart app
- Close all instances and reopen
- Check if app is running elsewhere

## Developer Notes

### Testing Updates

To test the update system:

1. **Create test release**:
   ```bash
   make release VERSION=1.0.1
   ```

2. **Install version 1.0.0** locally

3. **Start app** → Should detect 1.0.1

4. **Test update flow** → Install update

### Update Manifest Location

The update manifest is automatically generated and uploaded to:
```
https://github.com/The-Dev-Syndicate/UmbraRelay/releases/latest/download/latest.json
```

### Signing Keys

**Important**: 
- Keep private key secure (never commit to git)
- Store in secure location (password manager, encrypted)
- Back up private key (losing it means you can't sign updates)
- Public key is safe to commit (it's public by design)

## Future Enhancements

Potential improvements:
- **Silent updates**: Auto-install minor updates
- **Scheduled updates**: Install at specific times
- **Update channels**: Beta/stable channel support
- **Rollback**: Ability to revert to previous version
- **Delta updates**: Download only changes (smaller downloads)

## Related Documentation

- [Release Process](release-process.md) - How releases are created
- [Development Guide](development.md) - Development setup
- [Architecture](architecture.md) - System architecture overview
