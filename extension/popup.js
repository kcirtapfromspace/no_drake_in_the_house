/**
 * Popup Script for Kiro Extension
 * Handles the extension popup interface
 */

class PopupController {
  constructor() {
    this.currentTab = null;
    this.dnpList = [];
    this.stats = {
      hiddenToday: 0,
      skippedCount: 0
    };
    
    this.init();
  }

  async init() {
    try {
      // Get current tab info
      await this.getCurrentTab();
      
      // Load extension data
      await this.loadExtensionData();
      
      // Update UI
      this.updateUI();
      
      // Setup event listeners
      this.setupEventListeners();
      
      // Hide loading, show content
      document.getElementById('loading').classList.add('hidden');
      document.getElementById('main-content').classList.remove('hidden');
      
    } catch (error) {
      console.error('Failed to initialize popup:', error);
      this.showError('Failed to load extension data');
    }
  }

  async getCurrentTab() {
    const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
    this.currentTab = tabs[0];
  }

  async loadExtensionData() {
    // Load DNP list from storage
    const result = await chrome.storage.sync.get(['dnpList', 'authToken']);
    this.dnpList = result.dnpList || [];
    this.authToken = result.authToken;
    
    // Load stats from local storage
    const statsResult = await chrome.storage.local.get(['actionLog']);
    if (statsResult.actionLog) {
      this.calculateStats(statsResult.actionLog);
    }
  }

  calculateStats(actionLog) {
    const today = new Date().toDateString();
    
    this.stats.hiddenToday = actionLog.filter(entry => 
      new Date(entry.timestamp).toDateString() === today &&
      entry.action.type === 'content_hidden'
    ).length;
    
    this.stats.skippedCount = actionLog.filter(entry =>
      entry.action.type === 'track_skipped' || entry.action.type === 'track_auto_skipped'
    ).length;
  }

  updateUI() {
    // Update current site
    const siteElement = document.getElementById('current-site');
    const site = this.detectCurrentSite();
    siteElement.textContent = site;
    siteElement.className = `status-value ${site !== 'Unsupported' ? 'status-active' : 'status-inactive'}`;
    
    // Update server status
    const serverStatus = document.getElementById('server-status');
    if (this.authToken) {
      serverStatus.textContent = 'Connected';
      serverStatus.className = 'status-value status-active';
    } else {
      serverStatus.textContent = 'Not Logged In';
      serverStatus.className = 'status-value status-inactive';
    }
    
    // Update stats
    document.getElementById('blocked-count').textContent = this.dnpList.length;
    document.getElementById('hidden-today').textContent = this.stats.hiddenToday;
    document.getElementById('skipped-count').textContent = this.stats.skippedCount;
    
    // Update button states
    const addCurrentArtistBtn = document.getElementById('add-current-artist-btn');
    if (site === 'Unsupported') {
      addCurrentArtistBtn.disabled = true;
      addCurrentArtistBtn.innerHTML = '<span>🚫</span> Site Not Supported';
    }
  }

  detectCurrentSite() {
    if (!this.currentTab?.url) return 'Unknown';
    
    const url = this.currentTab.url;
    
    if (url.includes('open.spotify.com')) return 'Spotify';
    if (url.includes('music.youtube.com')) return 'YouTube Music';
    if (url.includes('music.apple.com')) return 'Apple Music';
    if (url.includes('tidal.com') || url.includes('listen.tidal.com')) return 'Tidal';
    
    return 'Unsupported';
  }

  setupEventListeners() {
    // Sync button
    document.getElementById('sync-btn').addEventListener('click', () => {
      this.syncWithServer();
    });
    
    // Add current artist button
    document.getElementById('add-current-artist-btn').addEventListener('click', () => {
      this.addCurrentArtist();
    });
    
    // View DNP list button
    document.getElementById('view-dnp-btn').addEventListener('click', () => {
      this.openDNPList();
    });
    
    // Settings button
    document.getElementById('settings-btn').addEventListener('click', () => {
      this.openSettings();
    });
    
    // Open dashboard link
    document.getElementById('open-dashboard').addEventListener('click', (e) => {
      e.preventDefault();
      this.openDashboard();
    });
  }

  async syncWithServer() {
    if (!this.authToken) {
      this.showError('Please log in to sync with server');
      return;
    }
    
    const syncBtn = document.getElementById('sync-btn');
    const originalText = syncBtn.innerHTML;
    syncBtn.innerHTML = '<div class="spinner"></div> Syncing...';
    syncBtn.disabled = true;
    
    try {
      // Send sync message to background script
      await chrome.runtime.sendMessage({
        type: 'SYNC_WITH_SERVER',
        authToken: this.authToken
      });
      
      // Reload data
      await this.loadExtensionData();
      this.updateUI();
      
      this.showSuccess('Synced successfully');
    } catch (error) {
      console.error('Sync failed:', error);
      this.showError('Sync failed. Please try again.');
    } finally {
      syncBtn.innerHTML = originalText;
      syncBtn.disabled = false;
    }
  }

  async addCurrentArtist() {
    const site = this.detectCurrentSite();
    if (site === 'Unsupported') return;
    
    const addBtn = document.getElementById('add-current-artist-btn');
    const originalText = addBtn.innerHTML;
    addBtn.innerHTML = '<div class="spinner"></div> Adding...';
    addBtn.disabled = true;
    
    try {
      // Send message to content script to get current artist
      const response = await chrome.tabs.sendMessage(this.currentTab.id, {
        type: 'GET_CURRENT_ARTIST'
      });
      
      if (response?.artistInfo) {
        // Add to DNP list
        await chrome.runtime.sendMessage({
          type: 'ADD_TO_DNP',
          artistInfo: response.artistInfo
        });
        
        // Reload data
        await this.loadExtensionData();
        this.updateUI();
        
        this.showSuccess(`Added ${response.artistInfo.name} to blocklist`);
      } else {
        this.showError('No artist detected on current page');
      }
    } catch (error) {
      console.error('Failed to add current artist:', error);
      this.showError('Failed to add artist. Make sure the page is loaded.');
    } finally {
      addBtn.innerHTML = originalText;
      addBtn.disabled = false;
    }
  }

  openDNPList() {
    // Create a simple DNP list view
    const popup = window.open('', 'dnpList', 'width=400,height=600,scrollbars=yes');
    
    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <title>Blocked Artists</title>
        <style>
          body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a; 
            color: white; 
            padding: 20px; 
            margin: 0;
          }
          .header { 
            border-bottom: 1px solid #333; 
            padding-bottom: 16px; 
            margin-bottom: 16px; 
          }
          .artist-item { 
            padding: 8px 0; 
            border-bottom: 1px solid #333; 
            display: flex;
            justify-content: space-between;
            align-items: center;
          }
          .artist-name { 
            font-weight: 500; 
          }
          .artist-source { 
            font-size: 12px; 
            color: #666; 
          }
          .remove-btn {
            background: #ff4444;
            color: white;
            border: none;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 11px;
            cursor: pointer;
          }
          .empty { 
            text-align: center; 
            color: #666; 
            padding: 40px 20px; 
          }
        </style>
      </head>
      <body>
        <div class="header">
          <h2>🚫 Blocked Artists (${this.dnpList.length})</h2>
        </div>
        ${this.dnpList.length === 0 ? 
          '<div class="empty">No artists blocked yet</div>' :
          this.dnpList.map(artist => `
            <div class="artist-item">
              <div>
                <div class="artist-name">${artist.name}</div>
                <div class="artist-source">${artist.source || 'Unknown source'}</div>
              </div>
              <button class="remove-btn" onclick="removeArtist('${artist.name}')">Remove</button>
            </div>
          `).join('')
        }
        <script>
          function removeArtist(artistName) {
            if (confirm('Remove ' + artistName + ' from blocklist?')) {
              // Send message to extension
              chrome.runtime.sendMessage({
                type: 'REMOVE_FROM_DNP',
                artistInfo: { name: artistName }
              }, () => {
                window.location.reload();
              });
            }
          }
        </script>
      </body>
      </html>
    `;
    
    popup.document.write(html);
    popup.document.close();
  }

  openSettings() {
    chrome.tabs.create({
      url: chrome.runtime.getURL('settings.html')
    });
  }

  openDashboard() {
    chrome.tabs.create({
      url: 'http://localhost:5000' // Frontend URL
    });
  }

  showError(message) {
    const errorElement = document.getElementById('error-message');
    errorElement.textContent = message;
    errorElement.classList.remove('hidden');
    
    setTimeout(() => {
      errorElement.classList.add('hidden');
    }, 5000);
  }

  showSuccess(message) {
    const successElement = document.getElementById('success-message');
    successElement.textContent = message;
    successElement.classList.remove('hidden');
    
    setTimeout(() => {
      successElement.classList.add('hidden');
    }, 3000);
  }
}

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new PopupController();
});