/**
 * Convert an ISO timestamp string to a human-readable relative time string.
 *
 * Examples: "Just now", "2 minutes ago", "3 hours ago", "Yesterday", "5 days ago", "Mar 10, 2026"
 */
export function timeAgo(isoString: string | null | undefined): string {
  if (!isoString) return 'Never';

  const date = new Date(isoString);
  if (Number.isNaN(date.getTime())) return 'Never';

  const now = Date.now();
  const diffMs = now - date.getTime();

  // Future dates or very recent (< 30s)
  if (diffMs < 30_000) return 'Just now';

  const diffSec = Math.floor(diffMs / 1_000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin} minute${diffMin === 1 ? '' : 's'} ago`;
  if (diffHr < 24) return `${diffHr} hour${diffHr === 1 ? '' : 's'} ago`;
  if (diffDay === 1) return 'Yesterday';
  if (diffDay < 7) return `${diffDay} days ago`;
  if (diffDay < 30) {
    const weeks = Math.floor(diffDay / 7);
    return `${weeks} week${weeks === 1 ? '' : 's'} ago`;
  }

  // Older than ~30 days: show a compact date
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}
