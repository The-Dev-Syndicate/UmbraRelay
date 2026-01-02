# Contributing to UmbraRelay

Thank you for your interest in contributing to UmbraRelay! We're excited to have you join us in making this project better.

## Getting Started

Before you start contributing, make sure you've set up your development environment. See the [Development Guide](documentation/technical/development.md) for detailed setup instructions.

## Contribution Workflow

### 1. Create an Issue

Before starting work on a feature or bug fix, please create an issue using the organization's issue templates. This helps us track work and avoid duplicate efforts.

### 2. Branch Naming

All branches must follow this naming convention:

- **Feature/Bug Fix**: `<issue_number>-<short-description>`
  - Example: `123-add-dark-mode` or `456-fix-rss-parsing-bug`
- **Temporary/Experimental**: `tmp-<description>` or `temp-<description>`
  - Example: `tmp-test-new-api`
- **No Issue**: `noissue-<description>`
  - Example: `noissue-update-docs`

**Important**: The issue number must match the GitHub issue number you're working on.

### 3. Commit Messages

All commit messages must start with the issue number in brackets:

- **With Issue**: `[#123] - Your commit message here`
- **Temporary**: `[TEMP] - Your commit message here`
- **No Issue**: `[noissue] - Your commit message here`

The commit message should be clear and descriptive of what the commit does.

### 4. Pull Requests

#### PR Title Format

PR titles must follow this format:

```
[#<issue_number>] - Brief description of changes
```

Example: `[#123] - Add dark mode support`

#### PR Description

Your PR description should include:

1. **Description**: Clear explanation of what changes you made and why
2. **Closing Keywords**: Use GitHub's closing keywords to link the PR to the issue:
   - `closes #123` - Closes the issue when merged
   - `fixes #123` - Fixes a bug
   - `resolves #123` - Resolves an issue
   - You can use multiple keywords if applicable

Example:
```markdown
## Description

This PR adds dark mode support to UmbraRelay by implementing a new theme system.

## Changes

- Added theme toggle in settings
- Implemented dark theme styles
- Updated theme persistence

## Testing

- Tested theme switching
- Verified theme persistence across app restarts

closes #123
```

3. **Testing**: Describe how you tested your changes
4. **Breaking Changes**: Note any breaking changes (if applicable)

### 5. Git Hook Helper (Optional)

We have an optional git hook that can help enforce the branch and commit naming conventions. It's completely optional—you don't need to use it if you prefer to manage commits manually.

If you'd like to use it, see the [Git Hooks documentation](documentation/technical/git-hooks.md) for setup instructions.

**Note**: This is just a helper tool. You're free to manage your commits however you prefer, as long as they follow the naming conventions above.

## Code Style

### Rust

- Follow Rust style guide
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`

### TypeScript/Vue

- Follow Vue 3 style guide
- Use consistent formatting

## Review Process

1. All PRs require at least one review before merging
2. Address any feedback or requested changes
3. Ensure all tests pass (if applicable)
4. Keep your branch up to date with the main branch

## Questions?

If you have questions or need help:

- Open an issue for discussion
- Check the [Development Guide](documentation/technical/development.md)
- Review the [Technical Documentation](documentation/technical/)

Thank you for contributing to UmbraRelay! 🎉

