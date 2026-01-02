/**
 * Utility functions for formatting and parsing data
 */

/**
 * Format a Unix timestamp to a localized date and time string
 * @param timestamp Unix timestamp in seconds
 * @returns Formatted date and time string
 */
export function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

/**
 * Format a Unix timestamp to a localized date and time string (alternative format)
 * @param timestamp Unix timestamp in seconds
 * @returns Formatted date and time string using toLocaleString
 */
export function formatDateLong(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

/**
 * Truncate a string to a maximum length, appending ellipsis if truncated
 * @param text Text to truncate
 * @param length Maximum length
 * @returns Truncated text with ellipsis if needed
 */
export function truncate(text: string, length: number): string {
  if (text.length <= length) return text;
  return text.substring(0, length) + '...';
}

/**
 * Strip HTML tags from a string and return plain text
 * @param html HTML string to strip
 * @returns Plain text without HTML tags
 */
export function stripHtml(html: string): string {
  if (!html) return '';
  const tmp = document.createElement('div');
  tmp.innerHTML = html;
  return tmp.textContent || tmp.innerText || '';
}

/**
 * Parse a comma-separated group string into an array of group names
 * @param groupString Comma-separated group string or null/undefined
 * @returns Array of trimmed group names
 */
export function parseGroups(groupString: string | null | undefined): string[] {
  if (!groupString) return [];
  return groupString.split(',').map(g => g.trim()).filter(g => g.length > 0);
}

/**
 * Format GitHub event type to user-friendly text
 * @param eventType Raw event type from GitHub API
 * @returns Human-readable event type description
 */
export function formatGitHubEventType(eventType: string): string {
  const eventMap: Record<string, string> = {
    'PullRequestEvent': 'Pull Request',
    'IssuesEvent': 'Issue',
    'PushEvent': 'Push',
    'CreateEvent': 'Created',
    'DeleteEvent': 'Deleted',
    'ForkEvent': 'Forked',
    'WatchEvent': 'Starred',
    'ReleaseEvent': 'Release',
    'CommitCommentEvent': 'Commit Comment',
    'IssueCommentEvent': 'Issue Comment',
    'PullRequestReviewEvent': 'PR Review',
    'PullRequestReviewCommentEvent': 'PR Review Comment',
    'GollumEvent': 'Wiki Page',
  };
  
  return eventMap[eventType] || eventType.replace(/([A-Z])/g, ' $1').trim();
}

/**
 * Format GitHub notification reason to user-friendly text
 * @param reason Raw reason from GitHub API
 * @returns Human-readable reason description
 */
export function formatGitHubNotificationReason(reason: string): string {
  const reasonMap: Record<string, string> = {
    'assign': 'You were assigned',
    'author': 'You created this',
    'comment': 'Someone commented',
    'ci_activity': 'CI/CD activity',
    'invitation': 'You were invited',
    'manual': 'You subscribed',
    'mention': 'You were mentioned',
    'review_requested': 'Review requested',
    'security_alert': 'Security alert',
    'state_change': 'Status changed',
    'subscribed': 'You\'re watching',
    'team_mention': 'Your team was mentioned',
  };
  
  return reasonMap[reason] || reason.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
}

/**
 * Format GitHub subject type to user-friendly text
 * @param subjectType Raw subject type from GitHub API
 * @returns Human-readable subject type
 */
export function formatGitHubSubjectType(subjectType: string): string {
  const typeMap: Record<string, string> = {
    'Issue': 'Issue',
    'PullRequest': 'Pull Request',
    'Commit': 'Commit',
    'Release': 'Release',
    'Discussion': 'Discussion',
    'RepositoryVulnerabilityAlert': 'Security Alert',
  };
  
  return typeMap[subjectType] || subjectType;
}

/**
 * Parse GitHub notification summary (format: "Issue - assign")
 * @param summary Summary string from GitHub notification
 * @returns Object with formatted type and reason, or null if not a notification format
 */
export function parseGitHubNotificationSummary(summary: string | null | undefined): { type: string; reason: string } | null {
  if (!summary) return null;
  
  // Pattern: "Issue - assign" or "PullRequest - review_requested"
  const match = summary.match(/^(.+?)\s*-\s*(.+)$/);
  if (match) {
    return {
      type: formatGitHubSubjectType(match[1].trim()),
      reason: formatGitHubNotificationReason(match[2].trim()),
    };
  }
  
  // Pattern: "Event type: PullRequestEvent"
  const eventMatch = summary.match(/^Event type:\s*(.+)$/);
  if (eventMatch) {
    return {
      type: formatGitHubEventType(eventMatch[1].trim()),
      reason: '',
    };
  }
  
  return null;
}

/**
 * Extract repository name from GitHub URL or category
 * @param item Item with category or URL
 * @returns Repository name (e.g., "owner/repo") or null
 */
export function extractGitHubRepo(item: { category?: string | null; url?: string }): string | null {
  // Try category first (usually contains repo name for GitHub items)
  if (item.category) {
    try {
      const categories = JSON.parse(item.category);
      if (Array.isArray(categories) && categories.length > 0) {
        const repo = categories[0];
        if (typeof repo === 'string' && repo.includes('/')) {
          return repo;
        }
      }
    } catch {
      // Not JSON, ignore
    }
  }
  
  // Try to extract from URL
  if (item.url) {
    const match = item.url.match(/github\.com\/([^\/]+\/[^\/]+)/);
    if (match) {
      return match[1];
    }
  }
  
  return null;
}

