/**
 * Tidal Content Script
 * Implements Tidal-specific artist detection and content filtering
 */

// Load base content script
const script = document.createElement('script');
script.src = chrome.runtime.getURL('content-scripts/base-content-script.js');
document.head.appendChild(script);

script.onload = () => {
  class TidalContentScript extends BaseContentScript {
    constructor() {
      const platformConfig = {
        platform: 'tidal',
        selectors: {
          artistLinks: 'a[href*="/artist/"], a[data-type="artist"]',
          trackRows: '[data-test="tracklist-row"], .tracklist-row',
          playlistTracks: '.playlist-tracklist [data-test="tracklist-row"]',
          artistCards: '[data-test="artist-card"], .artist-card',
          albumCards: '[data-test="album-card"], .album-card',
          searchResults: '.search-results',
          nowPlayingBar: '.playback-controls, .player-controls'
        },
        mediaSelectors: {
          playButton: '[data-test="play-button"], .play-button',
          nextButton: '[data-test="next-button"], .next-button',
          prevButton: '[data-test="previous-button"], .previous-button',
          progressBar: '.progress-bar'
        }
      };
      
      super(platformConfig);
    }

    extractArtistInfo(element) {
      // Try Tidal-specific extraction first
      const tidalInfo = this.extractTidalSpecific(element);
      if (tidalInfo) return tidalInfo;
      
      // Fall back to base implementation
      return super.extractArtistInfo(element);
    }

    extractTidalSpecific(element) {
      // Extract from Tidal track rows
      if (element.matches('[data-test="tracklist-row"], .tracklist-row')) {
        const artistLink = element.querySelector('a[href*="/artist/"], a[data-type="artist"]');
        if (artistLink) {
          const tidalId = this.extractTidalId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (tidalId && artistName) {
            return {
              name: artistName,
              tidalId: tidalId,
              source: 'tidal-track-row',
              element: element,
              isTrackRow: true
            };
          }
        }
      }

      // Extract from artist cards
      if (element.matches('[data-test="artist-card"], .artist-card')) {
        const link = element.querySelector('a[href*="/artist/"], a[data-type="artist"]');
        if (link) {
          const tidalId = this.extractTidalId(link.href);
          const artistName = element.querySelector('.artist-name, [data-test="artist-name"]')?.textContent?.trim();
          
          if (tidalId && artistName) {
            return {
              name: artistName,
              tidalId: tidalId,
              source: 'tidal-artist-card',
              element: element
            };
          }
        }
      }

      // Extract from direct artist links
      if (element.tagName.toLowerCase() === 'a' && 
          (element.href.includes('/artist/') || element.getAttribute('data-type') === 'artist')) {
        const tidalId = this.extractTidalId(element.href);
        const artistName = element.textContent?.trim();
        
        if (tidalId && artistName) {
          return {
            name: artistName,
            tidalId: tidalId,
            source: 'tidal-link',
            element: element
          };
        }
      }

      // Extract from now playing area
      if (element.closest('.playback-controls, .player-controls')) {
        const artistLink = element.querySelector('a[href*="/artist/"], a[data-type="artist"]') ||
                          (element.tagName.toLowerCase() === 'a' && 
                           (element.href.includes('/artist/') || element.getAttribute('data-type') === 'artist') ? element : null);
        
        if (artistLink) {
          const tidalId = this.extractTidalId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (tidalId && artistName) {
            return {
              name: artistName,
              tidalId: tidalId,
              source: 'tidal-now-playing',
              element: element,
              isNowPlaying: true
            };
          }
        }
      }

      // Extract from artist credits in track listings
      const creditElement = element.querySelector('.track-artist a[href*="/artist/"], .artist-credit a[href*="/artist/"]');
      if (creditElement) {
        const tidalId = this.extractTidalId(creditElement.href);
        const artistName = creditElement.textContent?.trim();
        
        if (tidalId && artistName) {
          return {
            name: artistName,
            tidalId: tidalId,
            source: 'tidal-credit',
            element: element,
            isTrackRow: true
          };
        }
      }

      return null;
    }

    extractTidalId(url) {
      if (!url) return null;
      
      // Extract from /artist/id format
      const match = url.match(/\/artist\/(\d+)/);
      if (match) {
        return match[1];
      }
      
      // Extract from /browse/artist/id format
      const browseMatch = url.match(/\/browse\/artist\/(\d+)/);
      if (browseMatch) {
        return browseMatch[1];
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
      
      // Hide the entire track row with Tidal styling
      element.style.cssText = `
        opacity: 0.3 !important;
        filter: grayscale(100%) !important;
        pointer-events: none !important;
        position: relative !important;
      `;

      // Disable play button if present
      const playButton = element.querySelector('[data-test="play-button"], .play-button');
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
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
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
        background: #1a1a1a;
        border: 1px solid #333;
        border-radius: 8px;
        padding: 8px 0;
        min-width: 160px;
        z-index: 1000;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      `;
      
      menu.innerHTML = `
        <div class="tidal-menu-item" data-action="play-once">
          <span>▶️</span> Play Once
        </div>
        <div class="tidal-menu-item" data-action="remove-from-dnp">
          <span>✅</span> Unblock Artist
        </div>
        <div class="tidal-menu-item" data-action="add-to-dnp">
          <span>🚫</span> Block Permanently
        </div>
      `;
      
      // Add menu styles
      const style = document.createElement('style');
      style.textContent = `
        .tidal-menu-item {
          padding: 10px 16px;
          color: #ccc;
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 13px;
          transition: background-color 0.2s;
        }
        .tidal-menu-item:hover {
          background: #333;
          color: white;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.closest('.tidal-menu-item')?.getAttribute('data-action');
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
      // Find and click the next button (Tidal specific)
      const nextButton = document.querySelector('[data-test="next-button"], .next-button');
      if (nextButton && !nextButton.disabled) {
        nextButton.click();
        this.showSkipNotification(artistInfo);
        this.logAction('track_auto_skipped', { artistInfo });
      }
    }

    extractTrackInfo(mediaElement) {
      // For Tidal, get current track info from player controls
      const playerControls = document.querySelector('.playback-controls, .player-controls');
      if (playerControls) {
        const artistLink = playerControls.querySelector('a[href*="/artist/"], a[data-type="artist"]');
        if (artistLink) {
          return this.extractTidalSpecific(artistLink);
        }
      }
      
      return super.extractTrackInfo(mediaElement);
    }

    showNotification(message, type = 'info') {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#00d4ff' : type === 'error' ? '#ff4444' : '#1a1a1a';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 12px 16px;
        border-radius: 8px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
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

    // Override to handle Tidal's specific DOM structure
    async scanExistingContent() {
      // Scan common Tidal containers
      const containers = [
        '[data-test="tracklist-row"]',
        '.tracklist-row',
        '[data-test="artist-card"]',
        '.artist-card',
        'a[href*="/artist/"]',
        'a[data-type="artist"]',
        '.playback-controls',
        '.player-controls',
        '.search-results'
      ];

      for (const selector of containers) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processElement(element);
        }
      }
    }

    // Handle Tidal's SPA navigation
    setupMutationObserver() {
      super.setupMutationObserver();
      
      // Tidal uses React Router, so we need to listen for route changes
      const originalPushState = history.pushState;
      const originalReplaceState = history.replaceState;
      
      history.pushState = function(...args) {
        originalPushState.apply(history, args);
        setTimeout(() => this.scanExistingContent(), 1000);
      }.bind(this);
      
      history.replaceState = function(...args) {
        originalReplaceState.apply(history, args);
        setTimeout(() => this.scanExistingContent(), 1000);
      }.bind(this);
      
      // Listen for popstate events
      window.addEventListener('popstate', () => {
        setTimeout(() => this.scanExistingContent(), 1000);
      });
      
      // Also listen for hash changes
      window.addEventListener('hashchange', () => {
        setTimeout(() => this.scanExistingContent(), 500);
      });
    }
  }

  // Initialize Tidal content script
  new TidalContentScript();
};