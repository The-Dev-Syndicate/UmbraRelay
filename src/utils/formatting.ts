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

