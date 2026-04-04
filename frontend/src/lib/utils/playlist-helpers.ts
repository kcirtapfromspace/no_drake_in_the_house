export function getGradeColor(g: string): string {
  switch (g) {
    case 'A+': return '#4ade80';
    case 'A':  return '#22c55e';
    case 'B':  return '#3b82f6';
    case 'C':  return '#eab308';
    case 'D':  return '#f97316';
    default:   return '#ef4444';
  }
}

export function getGradeGlow(g: string): string {
  switch (g) {
    case 'A+': case 'A': return '0 0 12px rgba(34,197,94,0.4)';
    case 'B':  return '0 0 12px rgba(59,130,246,0.35)';
    case 'C':  return '0 0 12px rgba(234,179,8,0.3)';
    case 'D':  return '0 0 12px rgba(249,115,22,0.3)';
    default:   return '0 0 12px rgba(239,68,68,0.35)';
  }
}

export function getGlowColor(g: string): string {
  switch (g) {
    case 'A+': return 'rgba(74, 222, 128, 0.35)';
    case 'A':  return 'rgba(34, 197, 94, 0.3)';
    case 'B':  return 'rgba(59, 130, 246, 0.3)';
    case 'C':  return 'rgba(234, 179, 8, 0.25)';
    case 'D':  return 'rgba(249, 115, 22, 0.25)';
    default:   return 'rgba(239, 68, 68, 0.3)';
  }
}

export function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = ((h << 5) - h + s.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

export function formatDurationMs(ms: number): string {
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}
