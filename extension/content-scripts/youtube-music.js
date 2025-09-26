/**
 * YouTube Music Content Script
 * Implements YouTube Music-specific artist detection and content filtering
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
        selectors: {
          artistLinks: 'a[href*="/channel/"], a[href*="/artist/"]',
          trackRows: 'ytmusic-responsive-list-item-renderer, ytmusic-playlist-shelf-renderer .ytmusic-responsive-list-item-renderer',
          playlistTracks: 'ytmusic-playlist-shelf-renderer ytmusic-responsive-list-item-renderer',
          artistCards: 'ytmusic-two-row-item-renderer[data-item-type="artist"]',
          albumCards: 'ytmusic-two-row-item-renderer[data-item-type="album"]',
          searchResults: '#contents ytmusic-shelf-renderer',
          nowPlayingBar: '.ytmusic-player-bar'
        },
        mediaSelectors: {
          playButton: '#play-pause-button',
          nextButton: '.next-button',
          prevButton: '.previous-button',
          progressBar: '#progress-bar'
        }
      };
      
      super(platformConfig);
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