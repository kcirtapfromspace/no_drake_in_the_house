/**
 * Apple Music Content Script
 * Implements Apple Music-specific artist detection and content filtering
 */

// Load base content script
const script = document.createElement('script');
script.src = chrome.runtime.getURL('content-scripts/base-content-script.js');
document.head.appendChild(script);

script.onload = () => {
  class AppleMusicContentScript extends BaseContentScript {
    constructor() {
      const platformConfig = {
        platform: 'apple-music',
        selectors: {
          artistLinks: 'a[href*="/artist/"]',
          trackRows: '[data-testid="track-list-item"], .songs-list-row',
          playlistTracks: '.songs-list .songs-list-row',
          artistCards: '[data-testid="artist-card"], .grid-item[data-type="artist"]',
          albumCards: '[data-testid="album-card"], .grid-item[data-type="album"]',
          searchResults: '.search-results',
          nowPlayingBar: '.web-chrome-playback-controls'
        },
        mediaSelectors: {
          playButton: '.web-chrome-playback-controls__playback-btn',
          nextButton: '.web-chrome-playback-controls__playback-btn--forward',
          prevButton: '.web-chrome-playback-controls__playback-btn--back',
          progressBar: '.web-chrome-playback-lcd__progress'
        }
      };
      
      super(platformConfig);
    }

    extractArtistInfo(element) {
      // Try Apple Music-specific extraction first
      const appleMusicInfo = this.extractAppleMusicSpecific(element);
      if (appleMusicInfo) return appleMusicInfo;
      
      // Fall back to base implementation
      return super.extractArtistInfo(element);
    }

    extractAppleMusicSpecific(element) {
      // Extract from Apple Music track rows
      if (element.matches('[data-testid="track-list-item"], .songs-list-row')) {
        const artistLink = element.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          const appleMusicId = this.extractAppleMusicId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (appleMusicId && artistName) {
            return {
              name: artistName,
              appleMusicId: appleMusicId,
              source: 'apple-music-track-row',
              element: element,
              isTrackRow: true
            };
          }
        }
      }

      // Extract from artist cards
      if (element.matches('[data-testid="artist-card"], .grid-item[data-type="artist"]')) {
        const link = element.querySelector('a[href*="/artist/"]');
        if (link) {
          const appleMusicId = this.extractAppleMusicId(link.href);
          const artistName = element.querySelector('.grid-item__title, [data-testid="artist-name"]')?.textContent?.trim();
          
          if (appleMusicId && artistName) {
            return {
              name: artistName,
              appleMusicId: appleMusicId,
              source: 'apple-music-artist-card',
              element: element
            };
          }
        }
      }

      // Extract from direct artist links
      if (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/')) {
        const appleMusicId = this.extractAppleMusicId(element.href);
        const artistName = element.textContent?.trim();
        
        if (appleMusicId && artistName) {
          return {
            name: artistName,
            appleMusicId: appleMusicId,
            source: 'apple-music-link',
            element: element
          };
        }
      }

      // Extract from now playing area
      if (element.closest('.web-chrome-playback-controls')) {
        const artistLink = element.querySelector('a[href*="/artist/"]') ||
                          (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/') ? element : null);
        
        if (artistLink) {
          const appleMusicId = this.extractAppleMusicId(artistLink.href);
          const artistName = artistLink.textContent?.trim();
          
          if (appleMusicId && artistName) {
            return {
              name: artistName,
              appleMusicId: appleMusicId,
              source: 'apple-music-now-playing',
              element: element,
              isNowPlaying: true
            };
          }
        }
      }

      // Extract from subtitle/byline elements
      const subtitleElement = element.querySelector('.songs-list-row__subtitle a[href*="/artist/"], .grid-item__subtitle a[href*="/artist/"]');
      if (subtitleElement) {
        const appleMusicId = this.extractAppleMusicId(subtitleElement.href);
        const artistName = subtitleElement.textContent?.trim();
        
        if (appleMusicId && artistName) {
          return {
            name: artistName,
            appleMusicId: appleMusicId,
            source: 'apple-music-subtitle',
            element: element,
            isTrackRow: true
          };
        }
      }

      return null;
    }

    extractAppleMusicId(url) {
      if (!url) return null;
      
      // Extract from /artist/name/id format
      const match = url.match(/\/artist\/[^/]+\/(\d+)/);
      if (match) {
        return match[1];
      }
      
      // Extract from direct ID format
      const idMatch = url.match(/\/artist\/(\d+)/);
      if (idMatch) {
        return idMatch[1];
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
      
      // Hide the entire track row with Apple Music styling
      element.style.cssText = `
        opacity: 0.3 !important;
        filter: grayscale(100%) !important;
        pointer-events: none !important;
        position: relative !important;
      `;

      // Disable play button if present
      const playButton = element.querySelector('.songs-list-row__play-button, [data-testid="play-button"]');
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
        font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
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
        background: #1c1c1e;
        border: 1px solid #38383a;
        border-radius: 8px;
        padding: 8px 0;
        min-width: 160px;
        z-index: 1000;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
      `;
      
      menu.innerHTML = `
        <div class="apple-music-menu-item" data-action="play-once">
          <span>▶️</span> Play Once
        </div>
        <div class="apple-music-menu-item" data-action="remove-from-dnp">
          <span>✅</span> Unblock Artist
        </div>
        <div class="apple-music-menu-item" data-action="add-to-dnp">
          <span>🚫</span> Block Permanently
        </div>
      `;
      
      // Add menu styles
      const style = document.createElement('style');
      style.textContent = `
        .apple-music-menu-item {
          padding: 10px 16px;
          color: #ebebf5;
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 13px;
          transition: background-color 0.2s;
        }
        .apple-music-menu-item:hover {
          background: #38383a;
          color: white;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.closest('.apple-music-menu-item')?.getAttribute('data-action');
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
      // Find and click the next button (Apple Music specific)
      const nextButton = document.querySelector('.web-chrome-playback-controls__playback-btn--forward');
      if (nextButton && !nextButton.disabled) {
        nextButton.click();
        this.showSkipNotification(artistInfo);
        this.logAction('track_auto_skipped', { artistInfo });
      }
    }

    extractTrackInfo(mediaElement) {
      // For Apple Music, get current track info from playback controls
      const playbackControls = document.querySelector('.web-chrome-playback-controls');
      if (playbackControls) {
        const artistLink = playbackControls.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          return this.extractAppleMusicSpecific(artistLink);
        }
      }
      
      return super.extractTrackInfo(mediaElement);
    }

    showNotification(message, type = 'info') {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#30d158' : type === 'error' ? '#ff453a' : '#1c1c1e';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 12px 16px;
        border-radius: 12px;
        font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
        font-size: 14px;
        z-index: 2147483647;
        animation: slideIn 0.3s ease-out;
        max-width: 300px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        backdrop-filter: blur(10px);
      `;
      
      notification.textContent = message;
      
      document.body.appendChild(notification);
      
      // Remove after 3 seconds
      setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-in';
        setTimeout(() => notification.remove(), 300);
      }, 3000);
    }

    // Override to handle Apple Music's specific DOM structure
    async scanExistingContent() {
      // Scan common Apple Music containers
      const containers = [
        '[data-testid="track-list-item"]',
        '.songs-list-row',
        '[data-testid="artist-card"]',
        '.grid-item[data-type="artist"]',
        'a[href*="/artist/"]',
        '.web-chrome-playback-controls',
        '.search-results'
      ];

      for (const selector of containers) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processElement(element);
        }
      }
    }

    // Handle Apple Music's SPA navigation
    setupMutationObserver() {
      super.setupMutationObserver();
      
      // Apple Music uses history API for navigation
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
    }
  }

  // Initialize Apple Music content script
  new AppleMusicContentScript();
};