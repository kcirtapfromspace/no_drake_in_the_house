/**
 * YouTube Music Content Script
 * Implements YouTube Music-specific artist detection and content filtering
 * ToS-compliant approach with preview-only mode and clear capability limitations
 */

// Load base content script
const script = document.createElement('script');
script.src = chrome.runtime.getURL('content-scripts/base-content-script.js');
document.head.appendChild(script);

script.onload = () => {
  class YouTubeMusicContentScript extends BaseContentScript {
    constructor() {
      const platformConfig = {
        platform: 'youtube-music',
        capabilities: {
          LIBRARY_PURGE: 'UNSUPPORTED',
          PLAYLIST_SCRUB: 'UNSUPPORTED', 
          ARTIST_BLOCK: 'UNSUPPORTED',
          RECOMMENDATION_FILTER: 'LIMITED',
          RADIO_SEED_FILTER: 'LIMITED',
          AUTOPLAY_SKIP: 'SUPPORTED',
          WEB_OVERLAY: 'SUPPORTED',
          FEATURED_ARTIST_DETECTION: 'LIMITED'
        },
        selectors: {
          artistLinks: 'a[href*="/channel/"], a[href*="/artist/"]',
          trackRows: 'ytmusic-responsive-list-item-renderer, ytmusic-playlist-shelf-renderer .ytmusic-responsive-list-item-renderer',
          playlistTracks: 'ytmusic-playlist-shelf-renderer ytmusic-responsive-list-item-renderer',
          artistCards: 'ytmusic-two-row-item-renderer[data-item-type="artist"]',
          albumCards: 'ytmusic-two-row-item-renderer[data-item-type="album"]',
          searchResults: '#contents ytmusic-shelf-renderer',
          nowPlayingBar: '.ytmusic-player-bar',
          recommendations: 'ytmusic-carousel-shelf-renderer, ytmusic-section-list-renderer',
          radioSeeds: '[data-testid="radio-builder"]'
        },
        mediaSelectors: {
          playButton: '#play-pause-button',
          nextButton: '.next-button, [aria-label*="Next"]',
          prevButton: '.previous-button, [aria-label*="Previous"]',
          progressBar: '#progress-bar'
        }
      };
      
      super(platformConfig);
      
      // YouTube Music specific initialization
      this.previewMode = true; // Always in preview mode due to API limitations
      this.tosCompliant = true;
      this.initializeYouTubeMusicFeatures();
    }

    async initializeYouTubeMusicFeatures() {
      // Show capability limitations to user
      this.showCapabilityNotification();
      
      // Set up export/import functionality
      this.setupDataExportImport();
      
      // Initialize recommendation filtering
      this.setupRecommendationFiltering();
      
      // Set up enhanced auto-skip with ToS compliance
      this.setupEnhancedAutoSkip();
    }

    showCapabilityNotification() {
      // Show a one-time notification about YouTube Music limitations
      chrome.storage.local.get(['ytmusic_notification_shown'], (result) => {
        if (!result.ytmusic_notification_shown) {
          this.showPersistentNotification(
            'YouTube Music Limitations',
            'Due to YouTube Music\'s Terms of Service, Kiro operates in preview-only mode. We can hide content and auto-skip tracks, but cannot modify your library directly. Use the export/import feature for manual synchronization.',
            'info',
            10000
          );
          chrome.storage.local.set({ ytmusic_notification_shown: true });
        }
      });
    }

    extractArtistInfo(element) {
      // Try YouTube Music-specific extraction first
      const ytMusicInfo = this.extractYouTubeMusicSpecific(element);
      if (ytMusicInfo) return ytMusicInfo;
      
      // Fall back to base implementation
      return super.extractArtistInfo(element);
    }

    extractYouTubeMusicSpecific(element) {
      // Extract from YouTube Music track rows
      if (element.tagName.toLowerCase() === 'ytmusic-responsive-list-item-renderer') {
        const artistLink = element.querySelector('a[href*="/channel/"], a[href*="/artist/"]');
        if (artistLink) {
          const channelId = this.extractChannelId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (channelId && artistName) {
            return {
              name: artistName,
              youtubeMusicId: channelId,
              source: 'ytmusic-track-row',
              element: element,
              isTrackRow: true
            };
          }
        }
      }

      // Extract from artist cards
      if (element.getAttribute('data-item-type') === 'artist') {
        const link = element.querySelector('a[href*="/channel/"], a[href*="/artist/"]');
        if (link) {
          const channelId = this.extractChannelId(link.href);
          const artistName = element.querySelector('.yt-simple-endpoint')?.textContent?.trim();
          
          if (channelId && artistName) {
            return {
              name: artistName,
              youtubeMusicId: channelId,
              source: 'ytmusic-artist-card',
              element: element
            };
          }
        }
      }

      // Extract from direct artist/channel links
      if (element.tagName.toLowerCase() === 'a' && 
          (element.href.includes('/channel/') || element.href.includes('/artist/'))) {
        const channelId = this.extractChannelId(element.href);
        const artistName = element.textContent?.trim();
        
        if (channelId && artistName) {
          return {
            name: artistName,
            youtubeMusicId: channelId,
            source: 'ytmusic-link',
            element: element
          };
        }
      }

      // Extract from now playing area
      if (element.closest('.ytmusic-player-bar')) {
        const artistLink = element.querySelector('a[href*="/channel/"], a[href*="/artist/"]') ||
                          (element.tagName.toLowerCase() === 'a' && 
                           (element.href.includes('/channel/') || element.href.includes('/artist/')) ? element : null);
        
        if (artistLink) {
          const channelId = this.extractChannelId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (channelId && artistName) {
            return {
              name: artistName,
              youtubeMusicId: channelId,
              source: 'ytmusic-now-playing',
              element: element,
              isNowPlaying: true
            };
          }
        }
      }

      // Extract from byline (secondary text in track listings)
      const bylineElement = element.querySelector('.secondary-flex-columns a[href*="/channel/"], .secondary-flex-columns a[href*="/artist/"]');
      if (bylineElement) {
        const channelId = this.extractChannelId(bylineElement.href);
        const artistName = bylineElement.textContent?.trim();
        
        if (channelId && artistName) {
          return {
            name: artistName,
            youtubeMusicId: channelId,
            source: 'ytmusic-byline',
            element: element,
            isTrackRow: true
          };
        }
      }

      return null;
    }

    extractChannelId(url) {
      if (!url) return null;
      
      // Extract from /channel/UC... format
      const channelMatch = url.match(/\/channel\/([^/?]+)/);
      if (channelMatch) {
        return channelMatch[1];
      }
      
      // Extract from /artist/... format (YouTube Music specific)
      const artistMatch = url.match(/\/artist\/([^/?]+)/);
      if (artistMatch) {
        return artistMatch[1];
      }
      
      return null;
    }

    hideElement(element, artistInfo) {
      if (this.blockedElements.has(element)) {
        return;
      }

      // Handle different types of elements differently
      if (artistInfo.isTrackRow) {
        this.hideTrackRow(element, artistInfo);
      } else if (artistInfo.isNowPlaying) {
        this.handleNowPlayingBlock(element, artistInfo);
      } else {
        // Default hiding for artist cards, links, etc.
        super.hideElement(element, artistInfo);
      }
    }

    hideTrackRow(element, artistInfo) {
      this.blockedElements.add(element);
      
      // Hide the entire track row with YouTube Music styling
      element.style.cssText = `
        opacity: 0.3 !important;
        filter: grayscale(100%) !important;
        pointer-events: none !important;
        position: relative !important;
      `;

      // Disable play button if present
      const playButton = element.querySelector('.play-button, [aria-label*="Play"]');
      if (playButton) {
        playButton.style.display = 'none';
      }

      // Add blocked indicator
      this.addTrackRowIndicator(element, artistInfo);
    }

    addTrackRowIndicator(element, artistInfo) {
      const indicator = document.createElement('div');
      indicator.style.cssText = `
        position: absolute;
        right: 10px;
        top: 50%;
        transform: translateY(-50%);
        background: rgba(0, 0, 0, 0.8);
        color: white;
        padding: 4px 8px;
        border-radius: 12px;
        font-size: 11px;
        font-weight: 500;
        z-index: 10;
        cursor: pointer;
        font-family: Roboto, Arial, sans-serif;
      `;
      
      indicator.textContent = '🚫 Blocked';
      indicator.title = `Blocked artist: ${artistInfo.name}`;
      
      // Add click handler for quick actions
      indicator.addEventListener('click', (e) => {
        e.stopPropagation();
        this.showQuickActions(element, artistInfo, indicator);
      });
      
      element.appendChild(indicator);
    }

    showQuickActions(element, artistInfo, indicator) {
      const menu = document.createElement('div');
      menu.style.cssText = `
        position: absolute;
        right: 0;
        top: 100%;
        background: #212121;
        border: 1px solid #404040;
        border-radius: 8px;
        padding: 8px 0;
        min-width: 160px;
        z-index: 1000;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        font-family: Roboto, Arial, sans-serif;
      `;
      
      menu.innerHTML = `
        <div class="ytmusic-menu-item" data-action="play-once">
          <span>▶️</span> Play Once
        </div>
        <div class="ytmusic-menu-item" data-action="remove-from-dnp">
          <span>✅</span> Unblock Artist
        </div>
        <div class="ytmusic-menu-item" data-action="add-to-dnp">
          <span>🚫</span> Block Permanently
        </div>
      `;
      
      // Add menu styles
      const style = document.createElement('style');
      style.textContent = `
        .ytmusic-menu-item {
          padding: 10px 16px;
          color: #aaa;
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 13px;
          transition: background-color 0.2s;
        }
        .ytmusic-menu-item:hover {
          background: #404040;
          color: white;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.closest('.ytmusic-menu-item')?.getAttribute('data-action');
        if (action) {
          await this.handleQuickAction(action, element, artistInfo);
          menu.remove();
          style.remove();
        }
      });
      
      // Close menu when clicking outside
      const closeMenu = (e) => {
        if (!menu.contains(e.target)) {
          menu.remove();
          style.remove();
          document.removeEventListener('click', closeMenu);
        }
      };
      setTimeout(() => document.addEventListener('click', closeMenu), 100);
      
      indicator.appendChild(menu);
    }

    async handleQuickAction(action, element, artistInfo) {
      switch (action) {
        case 'play-once':
          this.playOnce(element, artistInfo);
          break;
        case 'remove-from-dnp':
          await this.removeFromDNP(element, artistInfo);
          break;
        case 'add-to-dnp':
          await this.addToDNP(artistInfo);
          break;
      }
    }

    async addToDNP(artistInfo) {
      try {
        await new Promise((resolve) => {
          chrome.runtime.sendMessage({
            type: 'ADD_TO_DNP',
            artistInfo: artistInfo
          }, resolve);
        });
        
        this.logAction('added_to_dnp', { artistInfo });
        this.showNotification(`Added ${artistInfo.name} to blocklist`, 'success');
      } catch (error) {
        console.error('Failed to add to DNP:', error);
        this.showNotification('Failed to add to blocklist', 'error');
      }
    }

    handleNowPlayingBlock(element, artistInfo) {
      // For now playing, skip the current track
      this.skipCurrentTrack(artistInfo);
    }

    skipCurrentTrack(artistInfo) {
      // Find and click the next button (YouTube Music specific)
      const nextButton = document.querySelector('.next-button, [aria-label*="Next"]');
      if (nextButton && !nextButton.disabled) {
        nextButton.click();
        this.showSkipNotification(artistInfo);
        this.logAction('track_auto_skipped', { artistInfo });
      }
    }

    extractTrackInfo(mediaElement) {
      // For YouTube Music, get current track info from player bar
      const playerBar = document.querySelector('.ytmusic-player-bar');
      if (playerBar) {
        const artistLink = playerBar.querySelector('a[href*="/channel/"], a[href*="/artist/"]');
        if (artistLink) {
          return this.extractYouTubeMusicSpecific(artistLink);
        }
      }
      
      return super.extractTrackInfo(mediaElement);
    }

    setupDataExportImport() {
      // Add export/import controls to the page
      this.addExportImportUI();
      
      // Listen for export/import messages from popup
      chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
        if (message.type === 'EXPORT_YTMUSIC_DATA') {
          this.exportUserData().then(sendResponse);
          return true;
        } else if (message.type === 'IMPORT_YTMUSIC_DATA') {
          this.importUserData(message.data).then(sendResponse);
          return true;
        }
      });
    }

    addExportImportUI() {
      // Add a floating action button for export/import
      const fab = document.createElement('div');
      fab.id = 'kiro-ytmusic-fab';
      fab.style.cssText = `
        position: fixed;
        bottom: 20px;
        right: 20px;
        width: 56px;
        height: 56px;
        background: #ff0000;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        z-index: 2147483647;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        transition: all 0.3s ease;
        font-size: 24px;
        color: white;
      `;
      
      fab.innerHTML = '🎵';
      fab.title = 'Kiro - Export/Import YouTube Music Data';
      
      fab.addEventListener('click', () => {
        this.showExportImportMenu();
      });
      
      fab.addEventListener('mouseenter', () => {
        fab.style.transform = 'scale(1.1)';
      });
      
      fab.addEventListener('mouseleave', () => {
        fab.style.transform = 'scale(1)';
      });
      
      document.body.appendChild(fab);
    }

    showExportImportMenu() {
      const menu = document.createElement('div');
      menu.style.cssText = `
        position: fixed;
        bottom: 90px;
        right: 20px;
        background: #212121;
        border-radius: 8px;
        padding: 16px;
        min-width: 250px;
        z-index: 2147483647;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
        font-family: Roboto, Arial, sans-serif;
        color: white;
      `;
      
      menu.innerHTML = `
        <div class="kiro-menu-header">
          <h3 style="margin: 0 0 12px 0; font-size: 16px;">Kiro Data Sync</h3>
          <p style="margin: 0 0 16px 0; font-size: 12px; opacity: 0.8;">
            YouTube Music API limitations require manual sync
          </p>
        </div>
        <div class="kiro-menu-actions">
          <button class="kiro-menu-btn" data-action="export">
            📤 Export Blocked Content
          </button>
          <button class="kiro-menu-btn" data-action="import">
            📥 Import Blocklist
          </button>
          <button class="kiro-menu-btn" data-action="preview">
            👁️ Preview Mode Info
          </button>
          <button class="kiro-menu-btn secondary" data-action="close">
            ✕ Close
          </button>
        </div>
      `;
      
      // Add menu styles
      const style = document.createElement('style');
      style.textContent = `
        .kiro-menu-btn {
          display: block;
          width: 100%;
          padding: 10px 12px;
          margin: 4px 0;
          background: #404040;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 13px;
          text-align: left;
          transition: background-color 0.2s;
        }
        .kiro-menu-btn:hover {
          background: #505050;
        }
        .kiro-menu-btn.secondary {
          background: #666;
          margin-top: 12px;
        }
        .kiro-menu-btn.secondary:hover {
          background: #777;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.getAttribute('data-action');
        if (action) {
          switch (action) {
            case 'export':
              await this.handleExportData();
              break;
            case 'import':
              this.handleImportData();
              break;
            case 'preview':
              this.showPreviewModeInfo();
              break;
            case 'close':
              menu.remove();
              style.remove();
              break;
          }
        }
      });
      
      // Close menu when clicking outside
      const closeMenu = (e) => {
        if (!menu.contains(e.target) && e.target.id !== 'kiro-ytmusic-fab') {
          menu.remove();
          style.remove();
          document.removeEventListener('click', closeMenu);
        }
      };
      setTimeout(() => document.addEventListener('click', closeMenu), 100);
      
      document.body.appendChild(menu);
    }

    async handleExportData() {
      try {
        const exportData = await this.exportUserData();
        
        // Create download link
        const blob = new Blob([JSON.stringify(exportData, null, 2)], { 
          type: 'application/json' 
        });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = `kiro-youtube-music-export-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        this.showNotification('Export completed successfully', 'success');
      } catch (error) {
        console.error('Export failed:', error);
        this.showNotification('Export failed', 'error');
      }
    }

    handleImportData() {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.json';
      
      input.addEventListener('change', async (e) => {
        const file = e.target.files[0];
        if (file) {
          try {
            const text = await file.text();
            const data = JSON.parse(text);
            await this.importUserData(data);
            this.showNotification('Import completed successfully', 'success');
          } catch (error) {
            console.error('Import failed:', error);
            this.showNotification('Import failed - invalid file format', 'error');
          }
        }
      });
      
      input.click();
    }

    async exportUserData() {
      // Collect all visible content that would be blocked
      const blockedContent = {
        timestamp: new Date().toISOString(),
        platform: 'youtube-music',
        url: window.location.href,
        tracks: [],
        artists: [],
        recommendations: []
      };
      
      // Scan current page for blocked content
      const trackElements = document.querySelectorAll('ytmusic-responsive-list-item-renderer');
      for (const element of trackElements) {
        const artistInfo = this.extractArtistInfo(element);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            const trackTitle = element.querySelector('.title')?.textContent?.trim();
            blockedContent.tracks.push({
              artist: artistInfo.name,
              title: trackTitle,
              youtubeMusicId: artistInfo.youtubeMusicId,
              url: window.location.href
            });
          }
        }
      }
      
      // Scan for blocked artists
      const artistElements = document.querySelectorAll('ytmusic-two-row-item-renderer[data-item-type="artist"]');
      for (const element of artistElements) {
        const artistInfo = this.extractArtistInfo(element);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            blockedContent.artists.push({
              name: artistInfo.name,
              youtubeMusicId: artistInfo.youtubeMusicId
            });
          }
        }
      }
      
      return blockedContent;
    }

    async importUserData(data) {
      if (!data || !data.platform || data.platform !== 'youtube-music') {
        throw new Error('Invalid YouTube Music export data');
      }
      
      // Process imported blocklist data
      const importedArtists = data.artists || [];
      const importedTracks = data.tracks || [];
      
      // Send to background script for processing
      return new Promise((resolve) => {
        chrome.runtime.sendMessage({
          type: 'IMPORT_YTMUSIC_BLOCKLIST',
          data: {
            artists: importedArtists,
            tracks: importedTracks
          }
        }, resolve);
      });
    }

    setupRecommendationFiltering() {
      // Enhanced recommendation filtering with multiple strategies
      this.recommendationStrategies = {
        homeRecommendations: true,
        mixRecommendations: true,
        radioSeeds: true,
        relatedArtists: true,
        autoplayQueue: true
      };
      
      // Set up mutation observer for recommendation sections
      const recommendationObserver = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === 'childList') {
            this.processRecommendationMutations(mutation.addedNodes);
          }
        }
      });
      
      // Observe multiple recommendation containers
      const recommendationSelectors = [
        'ytmusic-carousel-shelf-renderer',
        'ytmusic-section-list-renderer', 
        'ytmusic-two-row-item-renderer',
        'ytmusic-responsive-list-item-renderer',
        '[data-testid="radio-builder"]',
        '.ytmusic-player-queue',
        '.mix-header',
        '.related-content'
      ];
      
      // Set up observers for each container type
      recommendationSelectors.forEach(selector => {
        const containers = document.querySelectorAll(selector);
        containers.forEach(container => {
          recommendationObserver.observe(container, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ['data-testid', 'aria-label']
          });
        });
      });
      
      // Set up "Not Interested" automation
      this.setupNotInterestedAutomation();
      
      // Set up radio seed filtering
      this.setupRadioSeedFiltering();
      
      // Initial scan of existing recommendations
      this.scanRecommendations();
      
      // Periodic rescan for dynamic content
      setInterval(() => {
        this.scanRecommendations();
      }, 10000);
    }

    async processRecommendationMutations(nodes) {
      for (const node of nodes) {
        if (node.nodeType === Node.ELEMENT_NODE) {
          await this.processRecommendationElement(node);
        }
      }
    }

    async processRecommendationElement(element) {
      // Check if this is a recommendation item
      if (element.matches('ytmusic-two-row-item-renderer, ytmusic-responsive-list-item-renderer')) {
        const artistInfo = this.extractArtistInfo(element);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            this.hideRecommendation(element, artistInfo);
          }
        }
      }
    }

    hideRecommendation(element, artistInfo) {
      // Hide recommendation with special styling
      element.style.cssText = `
        opacity: 0.2 !important;
        filter: grayscale(100%) blur(2px) !important;
        pointer-events: none !important;
        position: relative !important;
      `;
      
      // Add recommendation blocked indicator
      const indicator = document.createElement('div');
      indicator.style.cssText = `
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: rgba(0, 0, 0, 0.9);
        color: white;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 12px;
        font-weight: 500;
        z-index: 10;
        text-align: center;
        font-family: Roboto, Arial, sans-serif;
      `;
      
      indicator.innerHTML = `
        <div>🚫 Blocked Recommendation</div>
        <div style="font-size: 10px; opacity: 0.8; margin-top: 2px;">${artistInfo.name}</div>
      `;
      
      element.appendChild(indicator);
      
      this.logAction('recommendation_hidden', { artistInfo });
    }

    setupNotInterestedAutomation() {
      // ToS-compliant "Not Interested" automation
      this.notInterestedQueue = [];
      this.processingNotInterested = false;
      
      // Process queue periodically to avoid rate limiting
      setInterval(() => {
        this.processNotInterestedQueue();
      }, 5000);
    }

    async processNotInterestedQueue() {
      if (this.processingNotInterested || this.notInterestedQueue.length === 0) {
        return;
      }
      
      this.processingNotInterested = true;
      
      try {
        // Process one item at a time to be respectful
        const item = this.notInterestedQueue.shift();
        if (item) {
          await this.markAsNotInterested(item.element, item.artistInfo);
        }
      } catch (error) {
        console.error('Error processing not interested queue:', error);
      } finally {
        this.processingNotInterested = false;
      }
    }

    async markAsNotInterested(element, artistInfo) {
      // Look for "Not Interested" or menu button
      const menuButton = element.querySelector('[aria-label*="More"], [aria-label*="Menu"], .menu-button');
      
      if (menuButton) {
        // Simulate user interaction with delay
        setTimeout(() => {
          menuButton.click();
          
          // Look for "Not interested" option after menu opens
          setTimeout(() => {
            const notInterestedButton = document.querySelector('[aria-label*="Not interested"], [role="menuitem"]:contains("Not interested")');
            if (notInterestedButton) {
              notInterestedButton.click();
              this.logAction('marked_not_interested', { artistInfo });
              this.showNotification(`Marked ${artistInfo.name} as "Not Interested"`, 'success');
            }
          }, 500);
        }, 1000);
      }
    }

    setupRadioSeedFiltering() {
      // Monitor radio seed selection and filter blocked artists
      const radioObserver = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === 'childList') {
            this.processRadioSeedMutations(mutation.addedNodes);
          }
        }
      });
      
      // Observe radio builder and seed selection areas
      const radioContainers = document.querySelectorAll('[data-testid="radio-builder"], .radio-seeds, .seed-selection');
      radioContainers.forEach(container => {
        radioObserver.observe(container, {
          childList: true,
          subtree: true
        });
      });
      
      // Initial scan of radio seeds
      this.scanRadioSeeds();
    }

    async processRadioSeedMutations(nodes) {
      for (const node of nodes) {
        if (node.nodeType === Node.ELEMENT_NODE) {
          await this.processRadioSeedElement(node);
        }
      }
    }

    async processRadioSeedElement(element) {
      // Check if this is a radio seed element
      if (element.matches('[data-testid*="seed"], .seed-item, .radio-seed')) {
        const artistInfo = this.extractArtistInfo(element);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            this.hideRadioSeed(element, artistInfo);
          }
        }
      }
    }

    hideRadioSeed(element, artistInfo) {
      // Hide radio seed with special styling
      element.style.cssText = `
        opacity: 0.1 !important;
        filter: grayscale(100%) blur(1px) !important;
        pointer-events: none !important;
        position: relative !important;
      `;
      
      // Add blocked seed indicator
      const indicator = document.createElement('div');
      indicator.style.cssText = `
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(255, 0, 0, 0.8);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 11px;
        font-weight: bold;
        z-index: 10;
        border-radius: 4px;
      `;
      
      indicator.textContent = '🚫 BLOCKED SEED';
      element.appendChild(indicator);
      
      this.logAction('radio_seed_blocked', { artistInfo });
    }

    async scanRadioSeeds() {
      const radioSeedElements = document.querySelectorAll(
        '[data-testid*="seed"], .seed-item, .radio-seed, [data-testid="radio-builder"] *'
      );
      
      for (const element of radioSeedElements) {
        await this.processRadioSeedElement(element);
      }
    }

    async scanRecommendations() {
      // Enhanced recommendation scanning with multiple strategies
      const recommendationSelectors = [
        // Home page recommendations
        'ytmusic-carousel-shelf-renderer ytmusic-two-row-item-renderer',
        'ytmusic-section-list-renderer ytmusic-responsive-list-item-renderer',
        
        // Mix and playlist recommendations  
        '.mix-header ytmusic-two-row-item-renderer',
        '.related-content ytmusic-responsive-list-item-renderer',
        
        // Search suggestions
        'ytmusic-search-suggestion-renderer',
        
        // Autoplay queue
        '.ytmusic-player-queue ytmusic-player-queue-item',
        
        // Artist page recommendations
        '.artist-page-recommendations ytmusic-two-row-item-renderer'
      ];
      
      for (const selector of recommendationSelectors) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processRecommendationElement(element);
        }
      }
      
      // Scan for autoplay queue items
      await this.scanAutoplayQueue();
    }

    async scanAutoplayQueue() {
      const queueItems = document.querySelectorAll('.ytmusic-player-queue ytmusic-player-queue-item');
      
      for (const item of queueItems) {
        const artistInfo = this.extractArtistInfo(item);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            this.hideQueueItem(item, artistInfo);
          }
        }
      }
    }

    hideQueueItem(element, artistInfo) {
      // Hide queue item with special styling
      element.style.cssText = `
        opacity: 0.3 !important;
        filter: grayscale(100%) !important;
        position: relative !important;
      `;
      
      // Add remove button for queue items
      const removeButton = document.createElement('button');
      removeButton.style.cssText = `
        position: absolute;
        right: 8px;
        top: 50%;
        transform: translateY(-50%);
        background: #ff4444;
        color: white;
        border: none;
        border-radius: 50%;
        width: 24px;
        height: 24px;
        cursor: pointer;
        font-size: 12px;
        z-index: 10;
      `;
      
      removeButton.innerHTML = '×';
      removeButton.title = `Remove ${artistInfo.name} from queue`;
      
      removeButton.addEventListener('click', (e) => {
        e.stopPropagation();
        this.removeFromQueue(element, artistInfo);
      });
      
      element.appendChild(removeButton);
      
      this.logAction('queue_item_blocked', { artistInfo });
    }

    removeFromQueue(element, artistInfo) {
      // Try to find and click the remove button for this queue item
      const removeButton = element.querySelector('[aria-label*="Remove"], .remove-button');
      if (removeButton) {
        removeButton.click();
        this.showNotification(`Removed ${artistInfo.name} from queue`, 'success');
        this.logAction('queue_item_removed', { artistInfo });
      } else {
        // Fallback: just hide the element
        element.style.display = 'none';
        this.showNotification(`Hid ${artistInfo.name} from queue`, 'info');
      }
    }

    setupEnhancedAutoSkip() {
      // Enhanced auto-skip with ToS compliance checks
      this.skipAttempts = 0;
      this.maxSkipAttempts = 3; // Prevent infinite skip loops
      this.lastSkippedTrack = null;
      
      // Listen for track changes with debouncing
      let trackChangeTimeout;
      const handleTrackChange = () => {
        clearTimeout(trackChangeTimeout);
        trackChangeTimeout = setTimeout(() => {
          this.checkCurrentTrackForSkip();
        }, 1000);
      };
      
      // Multiple event listeners for robust track detection
      document.addEventListener('yt-navigate-finish', handleTrackChange);
      window.addEventListener('yt-player-state-changed', handleTrackChange);
      
      // Periodic check as fallback
      setInterval(() => {
        this.checkCurrentTrackForSkip();
      }, 5000);
    }

    async checkCurrentTrackForSkip() {
      const playerBar = document.querySelector('.ytmusic-player-bar');
      if (!playerBar) return;
      
      const artistInfo = this.extractArtistInfo(playerBar);
      if (!artistInfo) return;
      
      // Prevent skipping the same track multiple times
      const trackId = artistInfo.youtubeMusicId + '_' + (artistInfo.trackTitle || '');
      if (this.lastSkippedTrack === trackId) return;
      
      const isBlocked = await this.checkIfBlocked(artistInfo);
      if (isBlocked && this.skipAttempts < this.maxSkipAttempts) {
        this.performToSCompliantSkip(artistInfo, trackId);
      }
    }

    performToSCompliantSkip(artistInfo, trackId) {
      // ToS-compliant skip: simulate user interaction
      const nextButton = document.querySelector(this.mediaSelectors.nextButton);
      if (nextButton && !nextButton.disabled) {
        // Add a small delay to make it seem more natural
        setTimeout(() => {
          nextButton.click();
          this.lastSkippedTrack = trackId;
          this.skipAttempts++;
          
          // Reset skip attempts after successful skip
          setTimeout(() => {
            this.skipAttempts = 0;
          }, 10000);
          
          this.showSkipNotification(artistInfo, { tosCompliant: true });
          this.logAction('track_auto_skipped_tos_compliant', { artistInfo });
        }, 500);
      }
    }

    showPreviewModeInfo() {
      const modal = document.createElement('div');
      modal.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2147483647;
        font-family: Roboto, Arial, sans-serif;
      `;
      
      modal.innerHTML = `
        <div style="
          background: #212121;
          padding: 24px;
          border-radius: 12px;
          max-width: 600px;
          max-height: 80vh;
          overflow-y: auto;
          color: white;
          text-align: center;
        ">
          <h2 style="margin: 0 0 16px 0; color: #ff0000;">YouTube Music Limitations & Features</h2>
          
          <div style="text-align: left; margin: 16px 0;">
            <h3 style="color: #4caf50; margin: 12px 0 8px 0;">✅ What Kiro CAN do:</h3>
            <ul style="margin: 0; padding-left: 20px; line-height: 1.5;">
              <li><strong>Visual Content Filtering:</strong> Hide blocked artists with overlays</li>
              <li><strong>Auto-Skip Playback:</strong> Automatically skip blocked tracks</li>
              <li><strong>Recommendation Filtering:</strong> Hide blocked artists from recommendations</li>
              <li><strong>Radio Seed Filtering:</strong> Block artists from radio station seeds</li>
              <li><strong>Queue Management:</strong> Remove blocked tracks from autoplay queue</li>
              <li><strong>Export/Import:</strong> Manual synchronization workflows</li>
              <li><strong>"Not Interested" Automation:</strong> Mark blocked content as not interested</li>
            </ul>
            
            <h3 style="color: #ff9800; margin: 16px 0 8px 0;">⚠️ What Kiro CANNOT do (YouTube ToS):</h3>
            <ul style="margin: 0; padding-left: 20px; line-height: 1.5;">
              <li>Remove tracks from your library permanently via API</li>
              <li>Modify playlists or liked songs automatically</li>
              <li>Block artists at the YouTube account level</li>
              <li>Access private user data without consent</li>
              <li>Bypass YouTube's recommendation algorithms completely</li>
            </ul>
            
            <h3 style="color: #2196f3; margin: 16px 0 8px 0;">💡 Best Practices:</h3>
            <ol style="margin: 0; padding-left: 20px; line-height: 1.5;">
              <li><strong>Regular Exports:</strong> Export blocked content weekly</li>
              <li><strong>Manual Cleanup:</strong> Remove exported items from your library</li>
              <li><strong>Recommendation Training:</strong> Use "Not Interested" on blocked content</li>
              <li><strong>Radio Curation:</strong> Avoid blocked artists when creating radio stations</li>
              <li><strong>Playlist Review:</strong> Periodically check playlists for blocked content</li>
            </ol>
            
            <h3 style="color: #9c27b0; margin: 16px 0 8px 0;">🔧 Advanced Features:</h3>
            <ul style="margin: 0; padding-left: 20px; line-height: 1.5;">
              <li><strong>Smart Detection:</strong> Multiple artist detection strategies</li>
              <li><strong>Context Awareness:</strong> Different handling for different page types</li>
              <li><strong>Performance Optimized:</strong> Bloom filter for fast lookups</li>
              <li><strong>Privacy Focused:</strong> Local processing, minimal data collection</li>
            </ul>
          </div>
          
          <div style="margin-top: 20px;">
            <button onclick="this.parentElement.parentElement.remove()" style="
              background: #ff0000;
              color: white;
              border: none;
              padding: 12px 24px;
              border-radius: 6px;
              cursor: pointer;
              margin-right: 12px;
              font-size: 14px;
            ">Got it!</button>
            <button onclick="window.open('https://github.com/kiro-music/docs/youtube-music', '_blank')" style="
              background: #404040;
              color: white;
              border: none;
              padding: 12px 24px;
              border-radius: 6px;
              cursor: pointer;
              font-size: 14px;
            ">Learn More</button>
          </div>
        </div>
      `;
      
      document.body.appendChild(modal);
    }

    // Enhanced user education methods
    showLimitationWarning(context) {
      const warnings = {
        library: "YouTube Music API limitations prevent automatic library modifications. Use export/import for manual sync.",
        playlist: "Playlist modifications require manual action. Kiro can identify blocked content for you to remove.",
        recommendations: "Recommendation filtering is limited to visual hiding. Use 'Not Interested' for better algorithm training.",
        radio: "Radio seed filtering helps reduce blocked artists in generated stations, but cannot guarantee complete blocking."
      };
      
      const warning = warnings[context] || warnings.library;
      this.showNotification(warning, 'info');
    }

    showFeatureEducation(feature) {
      const education = {
        export: {
          title: "Export Feature",
          content: "Export creates a JSON file with all blocked content found on the current page. Use this to manually clean your library.",
          action: "Try exporting now to see what content would be blocked!"
        },
        import: {
          title: "Import Feature", 
          content: "Import allows you to restore your blocklist from a previous export or sync with other devices.",
          action: "Import a JSON file to update your blocklist."
        },
        autoSkip: {
          title: "Auto-Skip",
          content: "When a blocked track starts playing, Kiro automatically clicks the next button to skip it.",
          action: "This feature works automatically - no setup required!"
        },
        recommendations: {
          title: "Recommendation Filtering",
          content: "Kiro hides blocked artists from your home page, mixes, and related content sections.",
          action: "Blocked recommendations are grayed out with a 🚫 indicator."
        }
      };
      
      const info = education[feature];
      if (info) {
        this.showEducationalModal(info.title, info.content, info.action);
      }
    }

    showEducationalModal(title, content, action) {
      const modal = document.createElement('div');
      modal.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2147483647;
        font-family: Roboto, Arial, sans-serif;
      `;
      
      modal.innerHTML = `
        <div style="
          background: #212121;
          padding: 24px;
          border-radius: 8px;
          max-width: 400px;
          color: white;
          text-align: center;
        ">
          <h3 style="margin: 0 0 16px 0; color: #2196f3;">${title}</h3>
          <p style="margin: 0 0 16px 0; line-height: 1.5;">${content}</p>
          <p style="margin: 0 0 20px 0; font-style: italic; color: #aaa;">${action}</p>
          <button onclick="this.parentElement.parentElement.remove()" style="
            background: #2196f3;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
          ">Understood</button>
        </div>
      `;
      
      document.body.appendChild(modal);
      
      // Auto-remove after 10 seconds
      setTimeout(() => {
        if (modal.parentElement) {
          modal.remove();
        }
      }, 10000);
    }

    showNotification(message, type = 'info') {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#4caf50' : type === 'error' ? '#f44336' : '#323232';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 12px 16px;
        border-radius: 8px;
        font-family: Roboto, Arial, sans-serif;
        font-size: 14px;
        z-index: 2147483647;
        animation: slideIn 0.3s ease-out;
        max-width: 300px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      `;
      
      notification.textContent = message;
      
      document.body.appendChild(notification);
      
      // Remove after 3 seconds
      setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-in';
        setTimeout(() => notification.remove(), 300);
      }, 3000);
    }

    showPersistentNotification(title, message, type = 'info', duration = 5000) {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#4caf50' : type === 'error' ? '#f44336' : '#2196f3';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 16px;
        border-radius: 8px;
        font-family: Roboto, Arial, sans-serif;
        font-size: 14px;
        z-index: 2147483647;
        animation: slideIn 0.3s ease-out;
        max-width: 350px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      `;
      
      notification.innerHTML = `
        <div style="font-weight: bold; margin-bottom: 8px;">${title}</div>
        <div style="font-size: 13px; line-height: 1.4;">${message}</div>
        <button onclick="this.parentElement.remove()" style="
          position: absolute;
          top: 8px;
          right: 8px;
          background: none;
          border: none;
          color: white;
          cursor: pointer;
          font-size: 16px;
        ">×</button>
      `;
      
      document.body.appendChild(notification);
      
      // Auto-remove after duration
      setTimeout(() => {
        if (notification.parentElement) {
          notification.style.animation = 'slideOut 0.3s ease-in';
          setTimeout(() => notification.remove(), 300);
        }
      }, duration);
    }

    // Override to handle YouTube Music's specific DOM structure
    async scanExistingContent() {
      // Scan common YouTube Music containers
      const containers = [
        'ytmusic-responsive-list-item-renderer',
        'ytmusic-two-row-item-renderer',
        'a[href*="/channel/"]',
        'a[href*="/artist/"]',
        '.ytmusic-player-bar',
        '#contents ytmusic-shelf-renderer'
      ];

      for (const selector of containers) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processElement(element);
        }
      }
    }

    // Handle YouTube Music's SPA navigation
    setupMutationObserver() {
      super.setupMutationObserver();
      
      // YouTube Music uses custom navigation events
      window.addEventListener('yt-navigate-finish', () => {
        setTimeout(() => this.scanExistingContent(), 1000);
      });
      
      // Also listen for player state changes
      window.addEventListener('yt-player-state-changed', (e) => {
        if (e.detail && e.detail.playerState === 1) { // Playing
          setTimeout(() => this.checkCurrentTrack(), 500);
        }
      });
    }

    async checkCurrentTrack() {
      const playerBar = document.querySelector('.ytmusic-player-bar');
      if (playerBar) {
        const artistInfo = this.extractArtistInfo(playerBar);
        if (artistInfo) {
          const isBlocked = await this.checkIfBlocked(artistInfo);
          if (isBlocked) {
            this.skipCurrentTrack(artistInfo);
          }
        }
      }
    }
  }

  // Initialize YouTube Music content script
  new YouTubeMusicContentScript();
};