/**
 * Background Service Worker for Kiro Extension
 * Handles communication between content scripts and manages extension state
 */

// Import utilities
importScripts('utils/bloom-filter.js', 'utils/dnp-filter-manager.js');

class BackgroundService {
  constructor() {
    this.dnpFilterManager = null;
    this.lastSyncTime = 0;
    this.syncInterval = 5 * 60 * 1000; // 5 minutes
    
    this.setupMessageHandlers();
    this.setupStorageHandlers();
    this.initializeExtension();
  }

  setupMessageHandlers() {
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse);
      return true; // Keep message channel open for async responses
    });
  }

  setupStorageHandlers() {
    chrome.storage.onChanged.addListener((changes, namespace) => {
      if (namespace === 'sync' && changes.dnpList) {
        this.updateDNPCache(changes.dnpList.newValue);
      }
    });
  }

  async initializeExtension() {
    try {
      // Initialize DNP filter manager
      this.dnpFilterManager = new DNPFilterManager();
      await this.dnpFilterManager.init();
      
      // Sync with server if we have auth token
      const result = await chrome.storage.sync.get(['authToken']);
      if (result.authToken) {
        await this.dnpFilterManager.syncWithServer(result.authToken);
      }
      
      console.log('Kiro extension initialized with bloom filter');
    } catch (error) {
      console.error('Failed to initialize extension:', error);
      // Fallback initialization
      this.dnpFilterManager = new DNPFilterManager();
    }
  }

  async handleMessage(message, sender, sendResponse) {
    try {
      switch (message.type) {
        case 'CHECK_ARTIST_BLOCKED':
          const isBlocked = await this.checkArtistBlocked(message.artistInfo);
          sendResponse({ blocked: isBlocked, artistInfo: message.artistInfo });
          break;

        case 'GET_DNP_LIST':
          const dnpList = this.dnpFilterManager ? this.dnpFilterManager.fullDNPList : [];
          sendResponse({ dnpList });
          break;

        case 'ADD_TO_DNP':
          const addSuccess = await this.addToDNP(message.artistInfo);
          sendResponse({ success: addSuccess });
          break;

        case 'REMOVE_FROM_DNP':
          const removeSuccess = await this.removeFromDNP(message.artistInfo);
          sendResponse({ success: removeSuccess });
          break;

        case 'SYNC_WITH_SERVER':
          const syncSuccess = await this.syncWithServer(message.authToken);
          sendResponse({ success: syncSuccess });
          break;

        case 'GET_FILTER_STATS':
          const stats = this.dnpFilterManager ? this.dnpFilterManager.getStats() : null;
          sendResponse({ stats });
          break;

        case 'REBUILD_FILTER':
          const rebuildSuccess = this.dnpFilterManager ? await this.dnpFilterManager.rebuildFilter() : false;
          sendResponse({ success: rebuildSuccess });
          break;

        case 'LOG_ACTION':
          await this.logAction(message.action, sender.tab);
          sendResponse({ success: true });
          break;

        case 'EXPORT_YTMUSIC_DATA':
          const exportData = await this.exportYouTubeMusicData(sender.tab);
          sendResponse({ data: exportData });
          break;

        case 'IMPORT_YTMUSIC_DATA':
          const importSuccess = await this.importYouTubeMusicData(message.data);
          sendResponse({ success: importSuccess });
          break;

        case 'IMPORT_YTMUSIC_BLOCKLIST':
          const blocklistImportSuccess = await this.importYouTubeMusicBlocklist(message.data);
          sendResponse({ success: blocklistImportSuccess });
          break;

        default:
          console.warn('Unknown message type:', message.type);
          sendResponse({ error: 'Unknown message type' });
      }
    } catch (error) {
      console.error('Error handling message:', error);
      sendResponse({ error: error.message });
    }
  }

  async checkArtistBlocked(artistInfo) {
    if (!this.dnpFilterManager || !artistInfo) return false;

    // Use bloom filter for fast O(1) lookup
    return await this.dnpFilterManager.isBlocked(artistInfo);
  }

  async addToDNP(artistInfo) {
    try {
      if (!this.dnpFilterManager) return false;
      
      const success = await this.dnpFilterManager.addArtist(artistInfo);
      
      if (success) {
        // Sync with server if authenticated
        const authResult = await chrome.storage.sync.get(['authToken']);
        if (authResult.authToken) {
          await this.syncArtistWithServer(artistInfo, 'add', authResult.authToken);
        }
      }
      
      return success;
    } catch (error) {
      console.error('Failed to add artist to DNP:', error);
      return false;
    }
  }

  async removeFromDNP(artistInfo) {
    try {
      if (!this.dnpFilterManager) return false;
      
      const success = await this.dnpFilterManager.removeArtist(artistInfo);
      
      if (success) {
        // Sync with server if authenticated
        const authResult = await chrome.storage.sync.get(['authToken']);
        if (authResult.authToken) {
          await this.syncArtistWithServer(artistInfo, 'remove', authResult.authToken);
        }
      }
      
      return success;
    } catch (error) {
      console.error('Failed to remove artist from DNP:', error);
      return false;
    }
  }

  async syncWithServer(authToken) {
    try {
      if (!this.dnpFilterManager) return false;
      
      const success = await this.dnpFilterManager.syncWithServer(authToken);
      if (success) {
        this.lastSyncTime = Date.now();
        console.log('Synced DNP list with server using bloom filter');
      }
      
      return success;
    } catch (error) {
      console.error('Failed to sync with server:', error);
      return false;
    }
  }

  async syncArtistWithServer(artistInfo, action, authToken) {
    try {
      const endpoint = action === 'add' ? '/api/v1/dnp/add' : '/api/v1/dnp/remove';
      
      await fetch(`http://localhost:3000${endpoint}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ artist: artistInfo })
      });
    } catch (error) {
      console.error(`Failed to ${action} artist on server:`, error);
    }
  }

  async exportYouTubeMusicData(tab) {
    try {
      // Get current DNP list
      const dnpList = this.dnpFilterManager ? this.dnpFilterManager.fullDNPList : [];
      
      // Get YouTube Music specific data from storage
      const result = await chrome.storage.local.get(['ytmusic_blocked_content', 'ytmusic_export_history']);
      
      const exportData = {
        timestamp: new Date().toISOString(),
        platform: 'youtube-music',
        url: tab?.url,
        version: '1.0',
        capabilities: {
          LIBRARY_PURGE: 'UNSUPPORTED',
          PLAYLIST_SCRUB: 'UNSUPPORTED', 
          ARTIST_BLOCK: 'UNSUPPORTED',
          RECOMMENDATION_FILTER: 'LIMITED',
          RADIO_SEED_FILTER: 'LIMITED',
          AUTOPLAY_SKIP: 'SUPPORTED',
          WEB_OVERLAY: 'SUPPORTED'
        },
        dnpList: dnpList.map(artist => ({
          name: artist.name,
          externalIds: artist.externalIds || {},
          tags: artist.tags || [],
          addedAt: artist.addedAt
        })),
        blockedContent: result.ytmusic_blocked_content || [],
        exportHistory: result.ytmusic_export_history || [],
        instructions: {
          manual_sync_required: true,
          steps: [
            "1. Review the blocked content list below",
            "2. Manually remove these items from your YouTube Music library",
            "3. Use the import function to update your blocklist",
            "4. Repeat periodically to keep your library clean"
          ]
        }
      };
      
      // Update export history
      const exportHistory = result.ytmusic_export_history || [];
      exportHistory.push({
        timestamp: exportData.timestamp,
        itemCount: exportData.dnpList.length,
        url: tab?.url
      });
      
      // Keep only last 10 exports
      if (exportHistory.length > 10) {
        exportHistory.splice(0, exportHistory.length - 10);
      }
      
      await chrome.storage.local.set({ ytmusic_export_history: exportHistory });
      
      return exportData;
    } catch (error) {
      console.error('Failed to export YouTube Music data:', error);
      throw error;
    }
  }

  async importYouTubeMusicData(data) {
    try {
      if (!data || data.platform !== 'youtube-music') {
        throw new Error('Invalid YouTube Music export data');
      }
      
      // Store imported data
      await chrome.storage.local.set({
        ytmusic_imported_data: {
          ...data,
          importedAt: new Date().toISOString()
        }
      });
      
      // If DNP list is included, merge with existing
      if (data.dnpList && this.dnpFilterManager) {
        for (const artist of data.dnpList) {
          await this.dnpFilterManager.addArtist(artist);
        }
      }
      
      return true;
    } catch (error) {
      console.error('Failed to import YouTube Music data:', error);
      return false;
    }
  }

  async importYouTubeMusicBlocklist(data) {
    try {
      if (!data || (!data.artists && !data.tracks)) {
        throw new Error('Invalid blocklist data');
      }
      
      let importCount = 0;
      
      // Import artists
      if (data.artists && this.dnpFilterManager) {
        for (const artist of data.artists) {
          const artistInfo = {
            name: artist.name,
            youtubeMusicId: artist.youtubeMusicId,
            source: 'import',
            platform: 'youtube-music'
          };
          
          const success = await this.dnpFilterManager.addArtist(artistInfo);
          if (success) importCount++;
        }
      }
      
      // Import track artists (extract unique artists from tracks)
      if (data.tracks && this.dnpFilterManager) {
        const uniqueArtists = new Map();
        
        for (const track of data.tracks) {
          if (track.artist && !uniqueArtists.has(track.artist)) {
            uniqueArtists.set(track.artist, {
              name: track.artist,
              youtubeMusicId: track.youtubeMusicId,
              source: 'track-import',
              platform: 'youtube-music'
            });
          }
        }
        
        for (const artistInfo of uniqueArtists.values()) {
          const success = await this.dnpFilterManager.addArtist(artistInfo);
          if (success) importCount++;
        }
      }
      
      // Log import activity
      await this.logAction({
        type: 'ytmusic_blocklist_import',
        importCount,
        artistCount: data.artists?.length || 0,
        trackCount: data.tracks?.length || 0
      });
      
      return true;
    } catch (error) {
      console.error('Failed to import YouTube Music blocklist:', error);
      return false;
    }
  }

  async logAction(action, tab) {
    try {
      const logEntry = {
        action,
        timestamp: Date.now(),
        url: tab?.url,
        tabId: tab?.id
      };

      // Store locally for telemetry
      const result = await chrome.storage.local.get(['actionLog']);
      const actionLog = result.actionLog || [];
      actionLog.push(logEntry);
      
      // Keep only last 1000 entries
      if (actionLog.length > 1000) {
        actionLog.splice(0, actionLog.length - 1000);
      }
      
      await chrome.storage.local.set({ actionLog });
    } catch (error) {
      console.error('Failed to log action:', error);
    }
  }
}

// Initialize background service
new BackgroundService();