/**
 * DNP Filter Manager for Kiro Extension
 * Manages bloom filter for fast DNP list lookups with server sync
 */

class DNPFilterManager {
  constructor() {
    this.bloomFilter = null;
    this.fullDNPList = [];
    this.lastUpdateTime = 0;
    this.updateInterval = 5 * 60 * 1000; // 5 minutes
    this.maxCacheAge = 24 * 60 * 60 * 1000; // 24 hours
    
    this.init();
  }

  async init() {
    try {
      // Load cached filter and DNP list
      await this.loadFromStorage();
      
      // Initialize bloom filter if not loaded
      if (!this.bloomFilter) {
        this.bloomFilter = new BloomFilter(10000, 0.01);
        await this.rebuildFilter();
      }
      
      // Set up periodic updates
      this.setupPeriodicUpdates();
      
      console.log('DNP Filter Manager initialized');
    } catch (error) {
      console.error('Failed to initialize DNP Filter Manager:', error);
      this.bloomFilter = new BloomFilter(10000, 0.01);
    }
  }

  async loadFromStorage() {
    const result = await chrome.storage.local.get([
      'dnpBloomFilter',
      'dnpFilterTimestamp',
      'cachedDNPList'
    ]);
    
    // Load bloom filter
    if (result.dnpBloomFilter && result.dnpFilterTimestamp) {
      const age = Date.now() - result.dnpFilterTimestamp;
      
      if (age < this.maxCacheAge) {
        try {
          this.bloomFilter = BloomFilter.deserialize(result.dnpBloomFilter);
          this.lastUpdateTime = result.dnpFilterTimestamp;
          console.log('Loaded cached bloom filter');
        } catch (error) {
          console.warn('Failed to load cached bloom filter:', error);
        }
      }
    }
    
    // Load full DNP list
    if (result.cachedDNPList) {
      this.fullDNPList = result.cachedDNPList;
    }
  }

  async saveToStorage() {
    try {
      const filterData = this.bloomFilter.serialize();
      
      await chrome.storage.local.set({
        dnpBloomFilter: filterData,
        dnpFilterTimestamp: Date.now(),
        cachedDNPList: this.fullDNPList
      });
      
      console.log('Saved bloom filter to storage');
    } catch (error) {
      console.error('Failed to save bloom filter:', error);
    }
  }

  async rebuildFilter() {
    try {
      // Get latest DNP list from sync storage
      const result = await chrome.storage.sync.get(['dnpList']);
      const dnpList = result.dnpList || [];
      
      // Update full list
      this.fullDNPList = dnpList;
      
      // Rebuild bloom filter
      const expectedSize = Math.max(dnpList.length * 2, 1000);
      this.bloomFilter = new BloomFilter(expectedSize, 0.01);
      
      // Add all artists to filter
      for (const artist of dnpList) {
        this.addArtistToFilter(artist);
      }
      
      this.lastUpdateTime = Date.now();
      await this.saveToStorage();
      
      console.log(`Rebuilt bloom filter with ${dnpList.length} artists`);
      
      return true;
    } catch (error) {
      console.error('Failed to rebuild filter:', error);
      return false;
    }
  }

  addArtistToFilter(artist) {
    if (!artist || !this.bloomFilter) return;
    
    // Add multiple identifiers for better matching
    const identifiers = [
      artist.name?.toLowerCase().trim(),
      artist.spotifyId,
      artist.appleMusicId,
      artist.youtubeMusicId,
      artist.tidalId,
      artist.externalId
    ].filter(Boolean);
    
    for (const id of identifiers) {
      this.bloomFilter.add(id);
    }
    
    // Also add normalized variations
    if (artist.name) {
      const name = artist.name.toLowerCase().trim();
      this.bloomFilter.add(name);
      
      // Add without common prefixes/suffixes
      const cleanName = name
        .replace(/^the\s+/i, '')
        .replace(/\s+band$/i, '')
        .replace(/\s+group$/i, '');
      
      if (cleanName !== name) {
        this.bloomFilter.add(cleanName);
      }
    }
  }

  // Fast bloom filter check (O(1))
  mightBeBlocked(artistInfo) {
    if (!this.bloomFilter || !artistInfo) return false;
    
    const identifiers = [
      artistInfo.name?.toLowerCase().trim(),
      artistInfo.spotifyId,
      artistInfo.appleMusicId,
      artistInfo.youtubeMusicId,
      artistInfo.tidalId,
      artistInfo.externalId
    ].filter(Boolean);
    
    // Check if any identifier might be in the filter
    for (const id of identifiers) {
      if (this.bloomFilter.contains(id)) {
        return true; // Might be blocked (could be false positive)
      }
    }
    
    return false; // Definitely not blocked
  }

  // Definitive check against full list (slower but accurate)
  isDefinitelyBlocked(artistInfo) {
    if (!artistInfo) return false;
    
    return this.fullDNPList.some(blockedArtist => {
      // Exact name match
      if (artistInfo.name && blockedArtist.name &&
          artistInfo.name.toLowerCase().trim() === blockedArtist.name.toLowerCase().trim()) {
        return true;
      }
      
      // External ID matches
      const artistIds = [
        artistInfo.spotifyId,
        artistInfo.appleMusicId,
        artistInfo.youtubeMusicId,
        artistInfo.tidalId,
        artistInfo.externalId
      ].filter(Boolean);
      
      const blockedIds = [
        blockedArtist.spotifyId,
        blockedArtist.appleMusicId,
        blockedArtist.youtubeMusicId,
        blockedArtist.tidalId,
        blockedArtist.externalId
      ].filter(Boolean);
      
      return artistIds.some(id => blockedIds.includes(id));
    });
  }

  // Two-stage check: fast bloom filter first, then definitive check
  async isBlocked(artistInfo) {
    // Stage 1: Fast bloom filter check
    if (!this.mightBeBlocked(artistInfo)) {
      return false; // Definitely not blocked
    }
    
    // Stage 2: Definitive check (only for potential matches)
    return this.isDefinitelyBlocked(artistInfo);
  }

  async addArtist(artistInfo) {
    try {
      // Add to full list
      const exists = this.fullDNPList.some(artist => 
        artist.name?.toLowerCase() === artistInfo.name?.toLowerCase() ||
        (artistInfo.spotifyId && artist.spotifyId === artistInfo.spotifyId)
      );
      
      if (!exists) {
        this.fullDNPList.push({
          ...artistInfo,
          addedAt: Date.now(),
          source: 'extension'
        });
        
        // Add to bloom filter
        this.addArtistToFilter(artistInfo);
        
        // Update storage
        await chrome.storage.sync.set({ dnpList: this.fullDNPList });
        await this.saveToStorage();
        
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Failed to add artist:', error);
      return false;
    }
  }

  async removeArtist(artistInfo) {
    try {
      const originalLength = this.fullDNPList.length;
      
      this.fullDNPList = this.fullDNPList.filter(artist => 
        artist.name?.toLowerCase() !== artistInfo.name?.toLowerCase() &&
        (!artistInfo.spotifyId || artist.spotifyId !== artistInfo.spotifyId)
      );
      
      if (this.fullDNPList.length < originalLength) {
        // Rebuild filter since we can't remove from bloom filter
        await this.rebuildFilter();
        
        // Update sync storage
        await chrome.storage.sync.set({ dnpList: this.fullDNPList });
        
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Failed to remove artist:', error);
      return false;
    }
  }

  async syncWithServer(authToken) {
    try {
      // Check if we're online
      if (!navigator.onLine) {
        console.log('Offline - skipping server sync');
        return false;
      }
      
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 10000); // 10 second timeout
      
      const response = await fetch('http://localhost:3000/api/v1/dnp/list', {
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'Content-Type': 'application/json'
        },
        signal: controller.signal
      });
      
      clearTimeout(timeoutId);

      if (response.ok) {
        const data = await response.json();
        const serverList = data.artists || [];
        
        // Check if server list is different
        if (JSON.stringify(serverList) !== JSON.stringify(this.fullDNPList)) {
          this.fullDNPList = serverList;
          await this.rebuildFilter();
          
          // Store last successful sync time
          await chrome.storage.local.set({
            lastSuccessfulSync: Date.now(),
            serverSyncStatus: 'success'
          });
          
          console.log(`Synced ${serverList.length} artists from server`);
          return true;
        }
        
        // Update sync status even if no changes
        await chrome.storage.local.set({
          lastSuccessfulSync: Date.now(),
          serverSyncStatus: 'success'
        });
      } else {
        await chrome.storage.local.set({
          serverSyncStatus: 'error',
          lastSyncError: `HTTP ${response.status}`
        });
      }
      
      return false;
    } catch (error) {
      console.error('Failed to sync with server:', error);
      
      // Store error status
      await chrome.storage.local.set({
        serverSyncStatus: 'error',
        lastSyncError: error.message
      });
      
      return false;
    }
  }

  // Get offline status and sync information
  async getOfflineStatus() {
    const result = await chrome.storage.local.get([
      'lastSuccessfulSync',
      'serverSyncStatus',
      'lastSyncError'
    ]);
    
    const now = Date.now();
    const lastSync = result.lastSuccessfulSync || 0;
    const syncAge = now - lastSync;
    
    return {
      isOnline: navigator.onLine,
      lastSuccessfulSync: lastSync,
      syncAge: syncAge,
      syncStatus: result.serverSyncStatus || 'unknown',
      lastSyncError: result.lastSyncError,
      isStale: syncAge > this.maxCacheAge,
      canWorkOffline: this.fullDNPList.length > 0
    };
  }

  setupPeriodicUpdates() {
    // Check for updates periodically
    setInterval(async () => {
      const now = Date.now();
      
      // Check if filter needs rebuilding
      if (this.bloomFilter && this.bloomFilter.needsRebuild()) {
        console.log('Bloom filter needs rebuilding due to high false positive rate');
        await this.rebuildFilter();
      }
      
      // Check if we need to sync with server
      if (now - this.lastUpdateTime > this.updateInterval) {
        const authResult = await chrome.storage.sync.get(['authToken']);
        if (authResult.authToken) {
          await this.syncWithServer(authResult.authToken);
        }
      }
    }, 60000); // Check every minute
  }

  // Get performance statistics
  getStats() {
    const filterStats = this.bloomFilter ? this.bloomFilter.getStats() : null;
    
    return {
      totalArtists: this.fullDNPList.length,
      lastUpdateTime: this.lastUpdateTime,
      cacheAge: Date.now() - this.lastUpdateTime,
      bloomFilter: filterStats
    };
  }

  // Export DNP list
  exportList() {
    return {
      version: '1.0',
      exportDate: new Date().toISOString(),
      artists: this.fullDNPList,
      stats: this.getStats()
    };
  }

  // Import DNP list
  async importList(data) {
    if (!data || !data.artists || !Array.isArray(data.artists)) {
      throw new Error('Invalid import data');
    }
    
    this.fullDNPList = data.artists;
    await this.rebuildFilter();
    
    return true;
  }

  // Clear all data
  async clear() {
    this.fullDNPList = [];
    this.bloomFilter.clear();
    this.lastUpdateTime = 0;
    
    await chrome.storage.sync.set({ dnpList: [] });
    await chrome.storage.local.remove([
      'dnpBloomFilter',
      'dnpFilterTimestamp',
      'cachedDNPList'
    ]);
  }
}

// Export for use in extension
if (typeof module !== 'undefined' && module.exports) {
  module.exports = DNPFilterManager;
} else {
  window.DNPFilterManager = DNPFilterManager;
}