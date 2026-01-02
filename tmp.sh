#!/bin/bash
COMMIT_MSG_FILE=$1
COMMIT_SOURCE=$2
SHA1=$3

#
# --- 1. Skip rebase commits to preserve history ---
#
if [[ "$COMMIT_SOURCE" == "commit" && -n "$SHA1" ]]; then
  exit 0
fi

#
# --- 2. Detect current branch ---
#
branch_name=$(git symbolic-ref --short HEAD 2>/dev/null)

# Detached HEAD or CI: do nothing
if [ -z "$branch_name" ]; then
  exit 0
fi

#
# --- 3. Skip main/master ---
#
if [[ "$branch_name" == "main" || "$branch_name" == "master" ]]; then
  exit 0
fi

#
# --- 4. Read original commit message ---
#
original_msg=$(cat "$COMMIT_MSG_FILE")

# Extract first non-empty, non-comment line for prefix detection
first_line=$(echo "$original_msg" \
  | sed '/^[[:space:]]*$/d' \
  | sed '/^[[:space:]]*#/d' \
  | head -n1)

#
# --- 5. Determine prefix based on branch pattern ---
#
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

#
# --- 6. Validate issue number matches (if branch has issue number) ---
#
# If branch has an issue number, check if message has a different issue number
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

#
# --- 7. If the prefix already exists, skip ---
#
# Escape regex metacharacters in prefix
escaped_prefix=$(printf '%s\n' "$prefix" | sed -e 's/[][\.*^$()+?{}|]/\\&/g')

# Check if first line already starts with the prefix
if [[ -n "$first_line" && "$first_line" =~ ^${escaped_prefix}[[:space:]].* ]]; then
  exit 0
fi

#
# --- 8. Insert prefix only before the first meaningful line ---
#
echo "$original_msg" | awk -v prefix="$prefix" '
  BEGIN { prefixed = 0 }

  # Keep blank lines as-is
  /^[[:space:]]*$/ { print; next }

  # Keep comment lines (#...) as-is
  /^[[:space:]]*#/ { print; next }

  # First real content line → apply prefix
  prefixed == 0 {
    # Extract leading whitespace and content, then prepend prefix
    # Match leading whitespace
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
