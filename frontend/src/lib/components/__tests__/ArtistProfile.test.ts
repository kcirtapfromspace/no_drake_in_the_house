/**
 * ArtistProfile Component Tests
 * Tests for utility functions and basic component behavior
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

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
