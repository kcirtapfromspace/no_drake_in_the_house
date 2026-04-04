export function hideImgOnError(e: Event) {
  (e.currentTarget as HTMLImageElement).style.display = 'none';
}

interface ProviderBadge {
  name: string;
  color: string;
}

export function getProviderBadges(artist: any): ProviderBadge[] {
  const badges: ProviderBadge[] = [];
  if (artist?.external_ids?.spotify) badges.push({ name: 'Spotify', color: 'bg-green-900 text-green-300' });
  if (artist?.external_ids?.apple) badges.push({ name: 'Apple', color: 'bg-zinc-700 text-zinc-300' });
  if (artist?.external_ids?.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'bg-indigo-900 text-indigo-300' });
  return badges;
}

export function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString();
}
