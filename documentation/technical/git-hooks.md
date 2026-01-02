# Git Hooks

This document describes the optional git hook helper for UmbraRelay that enforces branch and commit naming conventions.

## Overview

The `prepare-commit-msg` hook automatically prefixes commit messages based on your branch name, ensuring consistency with our contribution guidelines.

## What It Does

The hook:

1. **Skips rebase commits** to preserve history
2. **Detects the current branch** name
3. **Skips main/master branches** (no prefix needed)
4. **Determines prefix** based on branch pattern:
   - `123-feature-name` → `[#123] -`
   - `tmp-description` or `temp-description` → `[TEMP] -`
   - `noissue-description` → `[noissue] -`
5. **Validates issue numbers** match between branch and commit message (if applicable)
6. **Adds prefix** to commit message if not already present

## Setup Instructions

### Option 1: Copy to .git/hooks

1. Copy the hook script to your `.git/hooks` directory:

```bash
cp documentation/technical/prepare-commit-msg.sh .git/hooks/prepare-commit-msg
chmod +x .git/hooks/prepare-commit-msg
```

### Option 2: Use from Documentation

If you prefer to keep it in the documentation folder:

```bash
ln -s ../../documentation/technical/prepare-commit-msg.sh .git/hooks/prepare-commit-msg
chmod +x .git/hooks/prepare-commit-msg
```

## Hook Script

The hook script is located at `documentation/technical/prepare-commit-msg.sh`. Here's what it does:

```bash
#!/bin/bash
COMMIT_MSG_FILE=$1
COMMIT_SOURCE=$2
SHA1=$3

# Skip rebase commits to preserve history
if [[ "$COMMIT_SOURCE" == "commit" && -n "$SHA1" ]]; then
  exit 0
fi

# Detect current branch
branch_name=$(git symbolic-ref --short HEAD 2>/dev/null)

# Detached HEAD or CI: do nothing
if [ -z "$branch_name" ]; then
  exit 0
fi

# Skip main/master
if [[ "$branch_name" == "main" || "$branch_name" == "master" ]]; then
  exit 0
fi

# Read original commit message
original_msg=$(cat "$COMMIT_MSG_FILE")

# Extract first non-empty, non-comment line for prefix detection
first_line=$(echo "$original_msg" \
  | sed '/^[[:space:]]*$/d' \
  | sed '/^[[:space:]]*#/d' \
  | head -n1)

# Determine prefix based on branch pattern
prefix=""

if [[ "$branch_name" =~ ^noissue ]]; then
  prefix="[noissue] -"

elif [[ "$branch_name" =~ ^(tmp|temp)- ]]; then
  prefix="[TEMP] -"

elif [[ "$branch_name" =~ ^([0-9]+)- ]]; then
  issue_number="${BASH_REMATCH[1]}"
  prefix="[#${issue_number}] -"
  
else
  # Unsupported branch naming scheme
  echo "ERROR: Branch name must follow one of these patterns:" >&2
  echo "  - <issue_number>-<description> (e.g., 2447-feature-name)" >&2
  echo "  - tmp-<description> or temp-<description>" >&2
  echo "  - noissue-<description>" >&2
  echo "  - main or master" >&2
  exit 1
fi

# Validate issue number matches (if branch has issue number)
if [[ "$branch_name" =~ ^([0-9]+)- ]]; then
  branch_issue_number="${BASH_REMATCH[1]}"
  
  # Check if message starts with [#<number>] pattern
  if [[ -n "$first_line" && "$first_line" =~ ^[[:space:]]*\[#([0-9]+)\][[:space:]]*- ]]; then
    message_issue_number="${BASH_REMATCH[1]}"
    
    # If numbers don't match, error out
    if [[ "$message_issue_number" != "$branch_issue_number" ]]; then
      echo "ERROR: Commit message issue number [#${message_issue_number}] does not match branch issue number [#${branch_issue_number}]" >&2
      echo "Branch: $branch_name" >&2
      echo "Expected prefix: [#${branch_issue_number}] -" >&2
      echo "Found prefix: [#${message_issue_number}] -" >&2
      exit 1
    fi
  fi
fi

# If the prefix already exists, skip
escaped_prefix=$(printf '%s\n' "$prefix" | sed -e 's/[][\.*^$()+?{}|]/\\&/g')

if [[ -n "$first_line" && "$first_line" =~ ^${escaped_prefix}[[:space:]].* ]]; then
  exit 0
fi

# Insert prefix only before the first meaningful line
echo "$original_msg" | awk -v prefix="$prefix" '
  BEGIN { prefixed = 0 }

  # Keep blank lines as-is
  /^[[:space:]]*$/ { print; next }

  # Keep comment lines (#...) as-is
  /^[[:space:]]*#/ { print; next }

  # First real content line → apply prefix
  prefixed == 0 {
    # Extract leading whitespace and content, then prepend prefix
    if (match($0, /^[[:space:]]*/)) {
      leading = substr($0, 1, RLENGTH)
      content = substr($0, RLENGTH + 1)
      print leading prefix " " content
    } else {
      print prefix " " $0
    }
    prefixed = 1
    next
  }

  # Print all other lines unchanged
  { print }
' > "$COMMIT_MSG_FILE"
```

## Usage Examples

### With Issue Number

```bash
# Branch: 123-add-feature
# Commit message: "Add new feature"
# Result: "[#123] - Add new feature"
```

### Temporary Branch

```bash
# Branch: tmp-test-something
# Commit message: "Test implementation"
# Result: "[TEMP] - Test implementation"
```

### No Issue

```bash
# Branch: noissue-update-docs
# Commit message: "Update documentation"
# Result: "[noissue] - Update documentation"
```

## Important Notes

- **This hook is completely optional**. You don't need to use it if you prefer to manage commits manually.
- The hook only helps enforce naming conventions—you still need to follow the contribution guidelines.
- If you encounter issues with the hook, you can always bypass it by using `git commit --no-verify` (though we prefer properly formatted commits).

## Troubleshooting

### Hook Not Running

- Make sure the file is executable: `chmod +x .git/hooks/prepare-commit-msg`
- Verify the file is in the correct location: `.git/hooks/prepare-commit-msg`

### Branch Name Errors

If you get an error about branch naming, make sure your branch follows one of the supported patterns:
- `<issue_number>-<description>`
- `tmp-<description>` or `temp-<description>`
- `noissue-<description>`
- `main` or `master`

### Issue Number Mismatch

If you get an error about issue number mismatch, ensure the issue number in your commit message matches the issue number in your branch name.

