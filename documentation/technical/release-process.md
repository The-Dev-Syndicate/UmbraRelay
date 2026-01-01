# Release Process

This document describes the process for creating and publishing releases of UmbraRelay.

## Overview

UmbraRelay uses a streamlined release process that:
- Creates and pushes a git tag
- Triggers automated cross-platform builds via GitHub Actions
- Creates a draft release on GitHub for review
- Automatically attaches build artifacts to the release

## Prerequisites

Before creating a release, ensure you have:

1. **Git access**: Push access to the repository
2. **GitHub CLI**: Installed and authenticated (`gh auth login`)
3. **Clean working directory**: All changes committed
4. **Version number**: Semantic version ready (e.g., `0.0.1`, `1.2.3`)

## Quick Start

### Development Mode

To start the development server:

```bash
make start-dev
```

This runs `npm run tauri dev` and starts the app with hot-reload enabled.

### Creating a Release

To create a new release:

```bash
make release VERSION=0.0.1
```

Replace `0.0.1` with your desired semantic version.

## Release Process Details

### Step 1: Confirmation Prompt

When you run `make release VERSION=X`, you'll see a confirmation prompt:

```
==========================================
  RELEASE WARNING
==========================================
This will:
  1. Create git tag: v0.0.1
  2. Push tag to origin
  3. Trigger GitHub Actions workflow
  4. Create a draft release on GitHub for review

Are you sure you want to proceed? (y/yes to continue, anything else to cancel)
```

**Important**: Type `y` or `yes` (case-insensitive) to proceed. Any other input will cancel the release process cleanly (no error).

### Step 2: Tag Creation

If confirmed, the process will:

1. **Create the tag**: `git tag v<VERSION>`
2. **Push the tag**: `git push origin v<VERSION>`

Example:
```bash
git tag v0.0.1
git push origin v0.0.1
```

### Step 3: GitHub Actions Workflow

Pushing the tag automatically triggers the `.github/workflows/release.yml` workflow, which:

#### Job 1: Create Release

1. **Checkout code**: Gets the latest code
2. **Setup Node.js**: Installs Node.js 20
3. **Extract version**: Reads version from `package.json`
4. **Check for changelog**: 
   - Looks for `CHANGELOG.md` in the repository
   - If found, checks if a section exists for the release version
   - Extracts the changelog section if available
5. **Create draft release**:
   - Creates a draft release on GitHub
   - Uses changelog content if available, otherwise auto-generates release notes
   - Release name: `UmbraRelay v<VERSION>`
   - Tag: `v<VERSION>`
6. **Get release ID**: Retrieves the release ID for artifact attachment

#### Job 2: Build Tauri (Cross-Platform)

Runs in parallel across three platforms:

- **macOS** (`macos-latest`): Builds for both `aarch64-apple-darwin` and `x86_64-apple-darwin`
- **Windows** (`windows-latest`): Builds for `x86_64-pc-windows-msvc`
- **Linux** (`ubuntu-latest`): Builds for `x86_64-unknown-linux-gnu`

Each build:
1. Checks out code
2. Sets up Node.js 20
3. Installs Rust stable with platform-specific targets
4. Installs system dependencies (Linux only)
5. Installs frontend dependencies (`npm install`)
6. Builds the Tauri app using `tauri-apps/tauri-action`
7. Attaches the build artifact to the draft release

### Step 4: Review and Publish

1. **Check GitHub Actions**: Monitor the workflow in the Actions tab
2. **Review draft release**: Go to the Releases section
3. **Verify artifacts**: Ensure all three platform builds are attached
4. **Review release notes**: Check auto-generated or changelog-based notes
5. **Publish**: Click "Publish release" when ready

## Release Artifacts

After the workflow completes, the draft release will contain:

- **macOS**: `.app` bundle (universal binary for Intel and Apple Silicon)
- **Windows**: `.msi` installer
- **Linux**: `.AppImage`

## Changelog Integration

### Automatic Changelog Detection

The workflow automatically detects and uses changelog entries if:

1. `CHANGELOG.md` exists in the repository root
2. A section exists for the release version

Supported formats:
```markdown
## v0.0.1
## [v0.0.1]
```

### Changelog Extraction

The workflow extracts the changelog section from the version header until:
- The next `##` header (next version), or
- End of file (if it's the latest version)

### Fallback

If no changelog is found, the workflow auto-generates release notes from:
- Commits since the last release
- Pull requests merged in this release

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

Examples:
- `0.0.1` - Initial release
- `0.1.0` - First feature release
- `1.0.0` - First stable release
- `1.2.3` - Patch release

## Troubleshooting

### Release ID Not Found

If you see:
```
gh: Not Found (HTTP 404)
Error: Process completed with exit code 1.
```

**Solution**: The workflow includes retry logic with exponential backoff (up to 10 attempts). This usually resolves timing issues. If it persists:

1. Check that the release was actually created in GitHub
2. Verify the tag name matches exactly
3. Check GitHub Actions logs for detailed error messages

### Workflow Not Triggered

If the workflow doesn't run after pushing a tag:

1. **Check tag format**: Must start with `v` (e.g., `v0.0.1`)
2. **Verify push**: Ensure `git push origin v<VERSION>` succeeded
3. **Check workflow file**: Ensure `.github/workflows/release.yml` exists
4. **Check permissions**: Verify GitHub Actions is enabled for the repository

### Build Failures

If a platform build fails:

1. **Check logs**: Review the specific platform's build logs
2. **Common issues**:
   - Missing system dependencies (Linux)
   - Rust toolchain issues
   - Node.js version mismatches
   - Network issues during dependency installation
3. **Retry**: Re-run the failed job from the Actions tab

### Draft Release Not Created

If the draft release isn't created:

1. **Check permissions**: Ensure the workflow has `contents: write` permission
2. **Check token**: Verify `GITHUB_TOKEN` is available (automatic in GitHub Actions)
3. **Check logs**: Review the `create-release` job logs
4. **Manual creation**: As a fallback, manually create a release and attach artifacts

## Manual Release Process

If you need to create a release manually:

1. **Create tag locally**:
   ```bash
   git tag v0.0.1
   git push origin v0.0.1
   ```

2. **Wait for builds**: Monitor GitHub Actions

3. **Create release manually** (if needed):
   - Go to GitHub Releases
   - Click "Draft a new release"
   - Select the tag
   - Add release notes
   - Attach build artifacts

## Best Practices

1. **Test before releasing**: Always test the app thoroughly before creating a release
2. **Update changelog**: Keep `CHANGELOG.md` up to date
3. **Semantic versioning**: Follow semantic versioning guidelines
4. **Review draft**: Always review the draft release before publishing
5. **Verify artifacts**: Ensure all platform builds completed successfully
6. **Clean commits**: Ensure your working directory is clean before releasing

## Makefile Reference

### Available Targets

```bash
make help          # Show available targets
make start-dev      # Start development server
make release VERSION=X  # Create and push release tag
```

### Makefile Location

The Makefile is located in the repository root: `Makefile`

## GitHub Actions Workflow

The release workflow is defined in: `.github/workflows/release.yml`

### Workflow Triggers

- **Automatic**: Push of a tag matching `v*` (e.g., `v0.0.1`)
- **Manual**: `workflow_dispatch` (but only runs if tag starts with `v`)

### Workflow Conditions

Both jobs only run if:
```yaml
if: startsWith(github.ref, 'refs/tags/v')
```

This ensures the workflow only runs for version tags, not on regular pushes to `main`.

## Related Documentation

- [Development Guide](development.md) - Development setup and workflow
- [Architecture](architecture.md) - System architecture
- [API Reference](api-reference.md) - Tauri command API

## Support

If you encounter issues with the release process:

1. Check the troubleshooting section above
2. Review GitHub Actions logs
3. Open an issue on GitHub with:
   - Error messages
   - Steps to reproduce
   - Relevant logs

