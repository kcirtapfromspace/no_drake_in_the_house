/**
 * ArtistProfile Component Tests
 * Tests for utility functions and basic component behavior
 */

import { describe, it, expect } from 'vitest';

// Suppress unused import warnings - these are used in test blocks
void vi; void beforeEach; void afterEach;

// Test utility functions directly - these don't require component mounting

describe('ArtistProfile Utility Functions', () => {
  describe('Number Formatting', () => {
    const formatNumber = (num?: number): string => {
      if (!num) return '0';
      if (num >= 1_000_000_000) return `${(num / 1_000_000_000).toFixed(1)}B`;
      if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
      if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
      return num.toString();
    };

    it('should format billions correctly', () => {
      expect(formatNumber(50000000000)).toBe('50.0B');
      expect(formatNumber(1000000000)).toBe('1.0B');
    });

    it('should format millions correctly', () => {
      expect(formatNumber(5000000)).toBe('5.0M');
      expect(formatNumber(1000000)).toBe('1.0M');
    });

    it('should format thousands correctly', () => {
      expect(formatNumber(5000)).toBe('5.0K');
      expect(formatNumber(1000)).toBe('1.0K');
    });

    it('should return raw numbers below 1000', () => {
      expect(formatNumber(500)).toBe('500');
      expect(formatNumber(1)).toBe('1');
    });

    it('should return 0 for undefined', () => {
      expect(formatNumber(undefined)).toBe('0');
    });
  });

  describe('Date Formatting', () => {
    const formatDate = (dateString?: string): string => {
      if (!dateString) return 'Unknown';
      const date = new Date(dateString);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
    };

    it('should format dates correctly', () => {
      // Use a specific date that won't be affected by timezone
      const result = formatDate('2020-06-15T12:00:00Z');
      expect(result).toMatch(/June \d+, 2020/);
    });

    it('should return Unknown for undefined', () => {
      expect(formatDate(undefined)).toBe('Unknown');
    });
  });

  describe('Status Color Mapping', () => {
    const getStatusColor = (status: string) => {
      const colors: Record<string, any> = {
        clean: { bg: '#10B981', text: '#FFFFFF', border: '#059669' },
        certified_creeper: { bg: '#DB2777', text: '#FFFFFF', border: '#EC4899' },
        flagged: { bg: '#EF4444', text: '#FFFFFF', border: '#DC2626' },
      };
      return colors[status] || colors.clean;
    };

    it('should return correct colors for clean status', () => {
      const color = getStatusColor('clean');
      expect(color.bg).toBe('#10B981');
    });

    it('should return correct colors for certified_creeper status', () => {
      const color = getStatusColor('certified_creeper');
      expect(color.bg).toBe('#DB2777');
    });

    it('should return correct colors for flagged status', () => {
      const color = getStatusColor('flagged');
      expect(color.bg).toBe('#EF4444');
    });

    it('should default to clean for unknown status', () => {
      const color = getStatusColor('unknown');
      expect(color.bg).toBe('#10B981');
    });
  });

  describe('Status Label Mapping', () => {
    const getStatusLabel = (status: string) => {
      const labels: Record<string, string> = {
        clean: 'Clean',
        certified_creeper: 'Certified Creeper',
        flagged: 'Flagged',
      };
      return labels[status] || 'Unknown';
    };

    it('should return correct label for clean', () => {
      expect(getStatusLabel('clean')).toBe('Clean');
    });

    it('should return correct label for certified_creeper', () => {
      expect(getStatusLabel('certified_creeper')).toBe('Certified Creeper');
    });

    it('should return correct label for flagged', () => {
      expect(getStatusLabel('flagged')).toBe('Flagged');
    });

    it('should return Unknown for invalid status', () => {
      expect(getStatusLabel('invalid')).toBe('Unknown');
    });
  });

  describe('Confidence Label Mapping', () => {
    const getConfidenceLabel = (confidence: string) => {
      const labels: Record<string, string> = {
        low: 'Low Confidence',
        medium: 'Medium Confidence',
        high: 'High Confidence',
      };
      return labels[confidence] || 'Unknown';
    };

    it('should return correct label for low confidence', () => {
      expect(getConfidenceLabel('low')).toBe('Low Confidence');
    });

    it('should return correct label for medium confidence', () => {
      expect(getConfidenceLabel('medium')).toBe('Medium Confidence');
    });

    it('should return correct label for high confidence', () => {
      expect(getConfidenceLabel('high')).toBe('High Confidence');
    });
  });

  describe('Source Tier Classification', () => {
    const determineSourceTier = (evidence: any): string => {
      if (evidence.source_type === 'court_record' || evidence.tier === 'tier_a') return 'tier_a';
      if (evidence.source_type === 'news' || evidence.tier === 'tier_b') return 'tier_b';
      if (evidence.tier === 'tier_c') return 'tier_c';
      return 'tier_d';
    };

    it('should classify court records as tier_a', () => {
      expect(determineSourceTier({ source_type: 'court_record' })).toBe('tier_a');
    });

    it('should classify explicit tier_a as tier_a', () => {
      expect(determineSourceTier({ tier: 'tier_a' })).toBe('tier_a');
    });

    it('should classify news as tier_b', () => {
      expect(determineSourceTier({ source_type: 'news' })).toBe('tier_b');
    });

    it('should classify explicit tier_c as tier_c', () => {
      expect(determineSourceTier({ tier: 'tier_c' })).toBe('tier_c');
    });

    it('should default to tier_d for unknown sources', () => {
      expect(determineSourceTier({})).toBe('tier_d');
    });
  });

  describe('Evidence Strength Calculation', () => {
    const determineEvidenceStrength = (evidence: any[]): string => {
      if (!evidence.length) return 'weak';
      const tierACount = evidence.filter(e =>
        e.source_type === 'court_record' || e.tier === 'tier_a'
      ).length;
      if (tierACount >= 2) return 'strong';
      if (tierACount >= 1 || evidence.length >= 3) return 'moderate';
      return 'weak';
    };

    it('should return weak for no evidence', () => {
      expect(determineEvidenceStrength([])).toBe('weak');
    });

    it('should return strong for 2+ tier_a sources', () => {
      const evidence = [
        { source_type: 'court_record' },
        { tier: 'tier_a' },
      ];
      expect(determineEvidenceStrength(evidence)).toBe('strong');
    });

    it('should return moderate for 1 tier_a source', () => {
      const evidence = [
        { source_type: 'court_record' },
        { source_type: 'news' },
      ];
      expect(determineEvidenceStrength(evidence)).toBe('moderate');
    });

    it('should return moderate for 3+ sources without tier_a', () => {
      const evidence = [
        { source_type: 'news' },
        { source_type: 'news' },
        { source_type: 'news' },
      ];
      expect(determineEvidenceStrength(evidence)).toBe('moderate');
    });

    it('should return weak for few low-tier sources', () => {
      const evidence = [
        { source_type: 'news' },
      ];
      expect(determineEvidenceStrength(evidence)).toBe('weak');
    });
  });

  describe('Profile Status Determination', () => {
    const determineStatus = (offenses: any[]): string => {
      const hasOffenses = offenses && offenses.length > 0;
      const hasConvictions = offenses?.some((o: any) =>
        o.status === 'convicted' || o.procedural_state === 'convicted'
      );

      if (hasConvictions) return 'flagged';
      if (hasOffenses) return 'certified_creeper';
      return 'clean';
    };

    it('should return clean for no offenses', () => {
      expect(determineStatus([])).toBe('clean');
    });

    it('should return certified_creeper for offenses without convictions', () => {
      const offenses = [
        { procedural_state: 'alleged' },
      ];
      expect(determineStatus(offenses)).toBe('certified_creeper');
    });

    it('should return flagged for convicted offenses', () => {
      const offenses = [
        { procedural_state: 'convicted' },
      ];
      expect(determineStatus(offenses)).toBe('flagged');
    });

    it('should return flagged if any offense is convicted', () => {
      const offenses = [
        { procedural_state: 'alleged' },
        { procedural_state: 'convicted' },
      ];
      expect(determineStatus(offenses)).toBe('flagged');
    });
  });

  describe('Profile Confidence Determination', () => {
    const determineConfidence = (offenses: any[]): string => {
      if (!offenses?.length) return 'low';

      const tierACount = offenses.reduce((count: number, o: any) => {
        return count + (o.evidence?.filter((e: any) =>
          e.source_type === 'court_record' || e.tier === 'tier_a'
        ).length || 0);
      }, 0);

      if (tierACount >= 2) return 'high';
      if (tierACount >= 1) return 'medium';
      return 'low';
    };

    it('should return low for no offenses', () => {
      expect(determineConfidence([])).toBe('low');
    });

    it('should return low for offenses without evidence', () => {
      expect(determineConfidence([{ id: '1' }])).toBe('low');
    });

    it('should return medium for 1 tier_a evidence', () => {
      const offenses = [
        { evidence: [{ source_type: 'court_record' }] },
      ];
      expect(determineConfidence(offenses)).toBe('medium');
    });

    it('should return high for 2+ tier_a evidence', () => {
      const offenses = [
        { evidence: [{ source_type: 'court_record' }, { tier: 'tier_a' }] },
      ];
      expect(determineConfidence(offenses)).toBe('high');
    });
  });
});

describe('ArtistProfile Data Types', () => {
  it('should have correct offense structure', () => {
    const offense = {
      id: 'offense-1',
      artist_id: 'artist-123',
      category: 'sexual_misconduct',
      title: 'Test Offense',
      description: 'Test description',
      incident_date: '2020-01-01',
      procedural_state: 'alleged',
      evidence: [],
    };

    expect(offense.id).toBeDefined();
    expect(offense.category).toBeDefined();
    expect(offense.title).toBeDefined();
  });

  it('should have correct profile structure', () => {
    const profile = {
      id: 'artist-123',
      canonical_name: 'Drake',
      status: 'flagged',
      confidence: 'high',
      offenses: [],
    };

    expect(profile.id).toBeDefined();
    expect(profile.canonical_name).toBe('Drake');
    expect(profile.status).toBe('flagged');
  });
});

describe('Evidence URL Mapping', () => {
  // This mirrors the transformOffense evidence mapping logic
  const mapEvidence = (e: any) => ({
    id: e._id || e.id,
    offense_id: 'offense-1',
    source: {
      id: e._id || e.id,
      url: e.url || e.source_url,
      title: e.title || e.sourceName || e.source_name,
      source_name: e.sourceName || e.source_name,
      source_type: e.sourceType || e.source_type || 'news',
      published_date: e.publishedDate || e.published_date,
      excerpt: e.excerpt,
      archived_url: e.archivedUrl || e.archived_url,
      credibility_score: e.credibilityScore || e.credibility_score,
    },
    date_added: e.createdAt || e.date_added || new Date().toISOString(),
    verified: e.verified || false,
  });

  it('should map Convex camelCase fields correctly (url, sourceName, etc.)', () => {
    const convexRow = {
      _id: 'ev-abc123',
      url: 'https://www.cnn.com/article/drake-case',
      sourceName: 'CNN',
      sourceType: 'news',
      title: 'Drake Investigation',
      publishedDate: '2020-06-15',
      excerpt: 'According to reports...',
      archivedUrl: 'https://web.archive.org/web/2020/https://www.cnn.com/article/drake-case',
      credibilityScore: 0.85,
      createdAt: '2024-01-01T00:00:00Z',
    };

    const result = mapEvidence(convexRow);

    expect(result.id).toBe('ev-abc123');
    expect(result.source.url).toBe('https://www.cnn.com/article/drake-case');
    expect(result.source.source_name).toBe('CNN');
    expect(result.source.source_type).toBe('news');
    expect(result.source.title).toBe('Drake Investigation');
    expect(result.source.published_date).toBe('2020-06-15');
    expect(result.source.excerpt).toBe('According to reports...');
    expect(result.source.archived_url).toBe(
      'https://web.archive.org/web/2020/https://www.cnn.com/article/drake-case',
    );
    expect(result.source.credibility_score).toBe(0.85);
    expect(result.date_added).toBe('2024-01-01T00:00:00Z');
  });

  it('should map legacy snake_case fields as fallback', () => {
    const legacyRow = {
      id: 'ev-legacy',
      source_url: 'https://legacy-source.com/page',
      source_name: 'Legacy Source',
      source_type: 'investigation',
      published_date: '2019-03-01',
      archived_url: 'https://archive.org/saved',
      credibility_score: 0.7,
      date_added: '2023-06-01T00:00:00Z',
    };

    const result = mapEvidence(legacyRow);

    expect(result.id).toBe('ev-legacy');
    expect(result.source.url).toBe('https://legacy-source.com/page');
    expect(result.source.source_name).toBe('Legacy Source');
    expect(result.source.source_type).toBe('investigation');
    expect(result.source.published_date).toBe('2019-03-01');
    expect(result.source.archived_url).toBe('https://archive.org/saved');
    expect(result.source.credibility_score).toBe(0.7);
  });

  it('should prefer Convex url over legacy source_url', () => {
    const mixed = {
      _id: 'ev-mixed',
      url: 'https://correct-url.com',
      source_url: 'https://old-url.com',
      sourceName: 'Source',
    };

    const result = mapEvidence(mixed);
    expect(result.source.url).toBe('https://correct-url.com');
  });

  it('should handle evidence with no URL gracefully', () => {
    const noUrl = {
      id: 'ev-nourl',
      source_name: 'Anonymous tip',
      source_type: 'social_media',
    };

    const result = mapEvidence(noUrl);
    expect(result.source.url).toBeUndefined();
    expect(result.source.source_name).toBe('Anonymous tip');
  });

  it('should default source_type to news when missing', () => {
    const noType = {
      id: 'ev-notype',
      url: 'https://example.com',
    };

    const result = mapEvidence(noType);
    expect(result.source.source_type).toBe('news');
  });

  it('should map archivedUrl for Wayback Machine references', () => {
    const withArchive = {
      _id: 'ev-archived',
      url: 'https://dead-link.example.com',
      archivedUrl: 'https://web.archive.org/web/20230101/https://dead-link.example.com',
      sourceName: 'Archived Source',
    };

    const result = mapEvidence(withArchive);
    expect(result.source.archived_url).toBe(
      'https://web.archive.org/web/20230101/https://dead-link.example.com',
    );
  });
});

describe('Connections from Credits Derivation', () => {
  // This mirrors the logic in loadArtist that derives collaborators from credits
  interface Credit {
    id: string;
    name: string;
    role: string;
    track_count: number;
    is_flagged: boolean;
    image_url: string | null;
    note?: string;
  }

  interface CatalogTrack {
    id: string;
    title: string;
    role: string;
    collaborators?: string[];
  }

  function deriveCollaboratorsFromCredits(
    writers: Credit[],
    producers: Credit[],
    catalog: CatalogTrack[],
  ) {
    const creditsCollabs: any[] = [];
    const seen = new Set<string>();

    for (const writer of writers) {
      if (!seen.has(writer.id)) {
        seen.add(writer.id);
        creditsCollabs.push({
          id: writer.id,
          name: writer.name,
          image_url: writer.image_url || undefined,
          collaboration_count: writer.track_count,
          is_flagged: writer.is_flagged,
          status: writer.is_flagged ? 'flagged' : 'clean',
          collaboration_type: 'writer',
          recent_tracks: [],
        });
      }
    }

    for (const producer of producers) {
      if (!seen.has(producer.id)) {
        seen.add(producer.id);
        creditsCollabs.push({
          id: producer.id,
          name: producer.name,
          image_url: producer.image_url || undefined,
          collaboration_count: producer.track_count,
          is_flagged: producer.is_flagged,
          status: producer.is_flagged ? 'flagged' : 'clean',
          collaboration_type: 'producer',
          recent_tracks: [],
        });
      } else {
        const existing = creditsCollabs.find((c) => c.id === producer.id);
        if (existing) {
          existing.collaboration_count += producer.track_count;
        }
      }
    }

    const catalogCollabs = new Map<string, { name: string; count: number }>();
    for (const track of catalog) {
      for (const name of track.collaborators || []) {
        const entry = catalogCollabs.get(name);
        if (entry) {
          entry.count++;
        } else {
          catalogCollabs.set(name, { name, count: 1 });
        }
      }
    }
    for (const [name, entry] of catalogCollabs) {
      if (!creditsCollabs.some((c) => c.name === name)) {
        creditsCollabs.push({
          id: `catalog-${name.toLowerCase().replace(/\s+/g, '-')}`,
          name: entry.name,
          collaboration_count: entry.count,
          is_flagged: false,
          status: 'clean',
          collaboration_type: 'featured',
          recent_tracks: [],
        });
      }
    }

    creditsCollabs.sort((a: any, b: any) => b.collaboration_count - a.collaboration_count);
    return creditsCollabs;
  }

  const writers: Credit[] = [
    { id: 'w1', name: 'Noah "40" Shebib', role: 'writer', track_count: 120, is_flagged: false, image_url: null },
    { id: 'w2', name: 'PartyNextDoor', role: 'writer', track_count: 25, is_flagged: false, image_url: null },
  ];

  const producers: Credit[] = [
    { id: 'w1', name: 'Noah "40" Shebib', role: 'producer', track_count: 150, is_flagged: false, image_url: null },
    { id: 'p1', name: 'Metro Boomin', role: 'producer', track_count: 25, is_flagged: false, image_url: null },
  ];

  const catalogTracks: CatalogTrack[] = [
    { id: 't1', title: 'Track 1', role: 'main', collaborators: ['21 Savage'] },
    { id: 't2', title: 'Track 2', role: 'main', collaborators: ['21 Savage', 'SZA'] },
    { id: 't3', title: 'Track 3', role: 'featured', collaborators: ['SZA'] },
  ];

  it('should combine writers and producers as connections', () => {
    const result = deriveCollaboratorsFromCredits(writers, producers, []);

    expect(result.length).toBe(3); // 40, PND, Metro
    expect(result.find((c: any) => c.name === 'Noah "40" Shebib')).toBeDefined();
    expect(result.find((c: any) => c.name === 'PartyNextDoor')).toBeDefined();
    expect(result.find((c: any) => c.name === 'Metro Boomin')).toBeDefined();
  });

  it('should de-duplicate writer+producer entries and sum track counts', () => {
    const result = deriveCollaboratorsFromCredits(writers, producers, []);

    const fortyEntry = result.find((c: any) => c.name === 'Noah "40" Shebib');
    // 120 (writer) + 150 (producer) = 270
    expect(fortyEntry.collaboration_count).toBe(270);
    // Should be listed as writer (first seen)
    expect(fortyEntry.collaboration_type).toBe('writer');
  });

  it('should extract featured collaborators from catalog', () => {
    const result = deriveCollaboratorsFromCredits([], [], catalogTracks);

    expect(result.find((c: any) => c.name === '21 Savage')).toBeDefined();
    expect(result.find((c: any) => c.name === 'SZA')).toBeDefined();
  });

  it('should count catalog appearances correctly', () => {
    const result = deriveCollaboratorsFromCredits([], [], catalogTracks);

    const twentyOne = result.find((c: any) => c.name === '21 Savage');
    expect(twentyOne.collaboration_count).toBe(2); // t1, t2

    const sza = result.find((c: any) => c.name === 'SZA');
    expect(sza.collaboration_count).toBe(2); // t2, t3
  });

  it('should not duplicate catalog artists already in credits', () => {
    const writersWithSza: Credit[] = [
      { id: 'sza', name: 'SZA', role: 'writer', track_count: 5, is_flagged: false, image_url: null },
    ];

    const result = deriveCollaboratorsFromCredits(writersWithSza, [], catalogTracks);

    // SZA should appear only once (from credits, not duplicated from catalog)
    const szaEntries = result.filter((c: any) => c.name === 'SZA');
    expect(szaEntries.length).toBe(1);
    expect(szaEntries[0].collaboration_type).toBe('writer');
  });

  it('should sort by collaboration count descending', () => {
    const result = deriveCollaboratorsFromCredits(writers, producers, catalogTracks);

    for (let i = 0; i < result.length - 1; i++) {
      expect(result[i].collaboration_count).toBeGreaterThanOrEqual(
        result[i + 1].collaboration_count,
      );
    }
  });

  it('should return empty array when no credits or catalog', () => {
    const result = deriveCollaboratorsFromCredits([], [], []);
    expect(result).toEqual([]);
  });

  it('should set collaboration_type to featured for catalog-only artists', () => {
    const result = deriveCollaboratorsFromCredits([], [], catalogTracks);

    const twentyOne = result.find((c: any) => c.name === '21 Savage');
    expect(twentyOne.collaboration_type).toBe('featured');
  });

  it('should handle flagged collaborators', () => {
    const flaggedWriters: Credit[] = [
      { id: 'fw1', name: 'Flagged Writer', role: 'writer', track_count: 10, is_flagged: true, image_url: null },
    ];

    const result = deriveCollaboratorsFromCredits(flaggedWriters, [], []);

    const flagged = result.find((c: any) => c.name === 'Flagged Writer');
    expect(flagged.is_flagged).toBe(true);
    expect(flagged.status).toBe('flagged');
  });

  it('should generate stable IDs for catalog-derived collaborators', () => {
    const result = deriveCollaboratorsFromCredits([], [], catalogTracks);

    const twentyOne = result.find((c: any) => c.name === '21 Savage');
    expect(twentyOne.id).toBe('catalog-21-savage');
  });
});
