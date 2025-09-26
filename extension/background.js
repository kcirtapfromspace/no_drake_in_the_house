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