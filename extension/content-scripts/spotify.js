/**
 * Spotify Content Script
 * Implements Spotify-specific artist detection and content filtering
 */

// Load base content script
const script = document.createElement('script');
script.src = chrome.runtime.getURL('content-scripts/base-content-script.js');
document.head.appendChild(script);

script.onload = () => {
  class SpotifyContentScript extends BaseContentScript {
    constructor() {
      const platformConfig = {
        platform: 'spotify',
        selectors: {
          artistLinks: 'a[href*="/artist/"]',
          trackRows: '[data-testid="tracklist-row"]',
          playlistTracks: '[data-testid="playlist-tracklist"] [data-testid="tracklist-row"]',
          artistCards: '[data-testid="artist-card"]',
          albumCards: '[data-testid="album-card"]',
          podcastCards: '[data-testid="podcast-card"]',
          searchResults: '[data-testid="search-results"]',
          nowPlayingBar: '[data-testid="now-playing-bar"]'
        },
        mediaSelectors: {
          playButton: '[data-testid="control-button-playpause"]',
          nextButton: '[data-testid="control-button-skip-forward"]',
          prevButton: '[data-testid="control-button-skip-back"]',
          progressBar: '[data-testid="progress-bar"]'
        }
      };
      
      super(platformConfig);
    }

    extractArtistInfo(element) {
      // Try Spotify-specific extraction first
      const spotifyInfo = this.extractSpotifySpecific(element);
      if (spotifyInfo) return spotifyInfo;
      
      // Fall back to base implementation
      return super.extractArtistInfo(element);
    }

    extractSpotifySpecific(element) {
      // Extract from Spotify URI in data attributes
      const uri = element.getAttribute('data-uri') || element.getAttribute('data-context-uri');
      if (uri && uri.includes('spotify:artist:')) {
        const spotifyId = uri.split('spotify:artist:')[1];
        const artistName = element.textContent?.trim() || 
                          element.querySelector('[data-testid="artist-name"]')?.textContent?.trim();
        
        if (spotifyId && artistName) {
          return {
            name: artistName,
            spotifyId: spotifyId,
            source: 'spotify-uri',
            element: element
          };
        }
      }

      // Extract from href attribute
      if (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/')) {
        const spotifyId = element.href.split('/artist/')[1]?.split('?')[0];
        const artistName = element.textContent?.trim();
        
        if (spotifyId && artistName) {
          return {
            name: artistName,
            spotifyId: spotifyId,
            source: 'spotify-href',
            element: element
          };
        }
      }

      // Extract from track row
      if (element.getAttribute('data-testid') === 'tracklist-row') {
        const artistLink = element.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          const spotifyId = artistLink.href.split('/artist/')[1]?.split('?')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (spotifyId && artistName) {
            return {
              name: artistName,
              spotifyId: spotifyId,
              source: 'track-row',
              element: element,
              isTrackRow: true
            };
          }
        }
      }

      // Extract from artist card
      if (element.getAttribute('data-testid') === 'artist-card') {
        const link = element.querySelector('a[href*="/artist/"]');
        if (link) {
          const spotifyId = link.href.split('/artist/')[1]?.split('?')[0];
          const artistName = element.querySelector('[data-testid="card-title"]')?.textContent?.trim();
          
          if (spotifyId && artistName) {
            return {
              name: artistName,
              spotifyId: spotifyId,
              source: 'artist-card',
              element: element
            };
          }
        }
      }

      // Extract from now playing bar
      if (element.closest('[data-testid="now-playing-bar"]')) {
        const artistLink = element.querySelector('a[href*="/artist/"]') || 
                          (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/') ? element : null);
        
        if (artistLink) {
          const spotifyId = artistLink.href.split('/artist/')[1]?.split('?')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (spotifyId && artistName) {
            return {
              name: artistName,
              spotifyId: spotifyId,
              source: 'now-playing',
              element: element,
              isNowPlaying: true
            };
          }
        }
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
      
      // Hide the entire track row
      element.style.cssText = `
        opacity: 0.3 !important;
        filter: grayscale(100%) !important;
        pointer-events: none !important;
        position: relative !important;
      `;

      // Disable play button
      const playButton = element.querySelector('[data-testid="play-button"]');
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
        padding: 2px 8px;
        border-radius: 12px;
        font-size: 11px;
        font-weight: 500;
        z-index: 10;
        cursor: pointer;
      `;
      
      indicator.textContent = '🚫 Blocked';
      indicator.title = `Blocked artist: ${artistInfo.name}`;
      
      // Add click handler for quick unblock
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
        background: #282828;
        border: 1px solid #404040;
        border-radius: 4px;
        padding: 8px 0;
        min-width: 150px;
        z-index: 1000;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
      `;
      
      menu.innerHTML = `
        <div class="menu-item" data-action="play-once">
          <span>▶️</span> Play Once
        </div>
        <div class="menu-item" data-action="remove-from-dnp">
          <span>✅</span> Unblock Artist
        </div>
        <div class="menu-item" data-action="add-to-dnp">
          <span>🚫</span> Block Permanently
        </div>
      `;
      
      // Add menu styles
      const style = document.createElement('style');
      style.textContent = `
        .menu-item {
          padding: 8px 16px;
          color: #b3b3b3;
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 13px;
        }
        .menu-item:hover {
          background: #404040;
          color: white;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.closest('.menu-item')?.getAttribute('data-action');
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
        
        // Show confirmation
        this.showNotification(`Added ${artistInfo.name} to blocklist`, 'success');
      } catch (error) {
        console.error('Failed to add to DNP:', error);
        this.showNotification('Failed to add to blocklist', 'error');
      }
    }

    handleNowPlayingBlock(element, artistInfo) {
      // For now playing, we want to skip the track immediately
      this.skipCurrentTrack(artistInfo);
    }

    skipCurrentTrack(artistInfo) {
      // Find and click the next button
      const nextButton = document.querySelector('[data-testid="control-button-skip-forward"]');
      if (nextButton && !nextButton.disabled) {
        nextButton.click();
        this.showSkipNotification(artistInfo);
        this.logAction('track_auto_skipped', { artistInfo });
      }
    }

    extractTrackInfo(mediaElement) {
      // For Spotify, we can get current track info from the now playing bar
      const nowPlayingBar = document.querySelector('[data-testid="now-playing-bar"]');
      if (nowPlayingBar) {
        const artistLink = nowPlayingBar.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          return this.extractSpotifySpecific(artistLink);
        }
      }
      
      return super.extractTrackInfo(mediaElement);
    }

    showNotification(message, type = 'info') {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#1db954' : type === 'error' ? '#e22134' : '#1e1e1e';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 12px 16px;
        border-radius: 6px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        font-size: 14px;
        z-index: 2147483647;
        animation: slideIn 0.3s ease-out;
        max-width: 300px;
      `;
      
      notification.textContent = message;
      
      document.body.appendChild(notification);
      
      // Remove after 3 seconds
      setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-in';
        setTimeout(() => notification.remove(), 300);
      }, 3000);
    }

    // Override to handle Spotify's specific DOM structure
    async scanExistingContent() {
      // Scan common Spotify containers
      const containers = [
        '[data-testid="tracklist-row"]',
        '[data-testid="artist-card"]',
        '[data-testid="album-card"]',
        'a[href*="/artist/"]',
        '[data-testid="search-results"]',
        '[data-testid="now-playing-bar"]'
      ];

      for (const selector of containers) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processElement(element);
        }
      }
    }
  }

  // Initialize Spotify content script
  new SpotifyContentScript();
};