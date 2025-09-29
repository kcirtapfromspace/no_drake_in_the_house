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
          trackRows: '.songs-list-row, .tracklist__item',
          playlistTracks: '.songs-list .songs-list-row',
          artistCards: '.grid-item--artist, .artist-lockup',
          albumCards: '.grid-item--album, .album-lockup',
          searchResults: '.search-results, .shelf-grid',
          nowPlayingBar: '.web-chrome-playback-controls, .playback-controls'
        },
        mediaSelectors: {
          playButton: '.web-chrome-playback-controls__playback-btn, .playback-controls__playback-btn',
          nextButton: '.web-chrome-playback-controls__next-btn, .playback-controls__next-btn',
          prevButton: '.web-chrome-playback-controls__previous-btn, .playback-controls__previous-btn',
          progressBar: '.web-chrome-playback-controls__progress, .playback-controls__progress'
        }
      };
      
      super(platformConfig);
      
      // Apple Music specific initialization
      this.initAppleMusicSpecific();
    }

    initAppleMusicSpecific() {
      // Apple Music uses a lot of dynamic content loading
      // Set up additional observers for their specific patterns
      this.setupAppleMusicObservers();
      
      // Apple Music has limited API access, so we focus on visual blocking
      this.showLimitationNotice();
    }

    setupAppleMusicObservers() {
      // Watch for Apple Music's specific content containers
      const appleMusicContainers = [
        '.web-chrome-playback-controls',
        '.songs-list',
        '.shelf-grid',
        '.search-results',
        '.artist-header',
        '.album-header'
      ];

      appleMusicContainers.forEach(selector => {
        const container = document.querySelector(selector);
        if (container) {
          const observer = new MutationObserver((mutations) => {
            mutations.forEach(mutation => {
              if (mutation.type === 'childList') {
                this.processMutations(mutation.addedNodes);
              }
            });
          });
          
          observer.observe(container, {
            childList: true,
            subtree: true
          });
        }
      });
    }

    showLimitationNotice() {
      // Show a notice about Apple Music's limitations
      const notice = document.createElement('div');
      notice.id = 'kiro-apple-music-notice';
      notice.style.cssText = `
        position: fixed;
        top: 10px;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(0, 0, 0, 0.9);
        color: white;
        padding: 8px 16px;
        border-radius: 6px;
        font-size: 12px;
        z-index: 2147483647;
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        max-width: 400px;
        text-align: center;
      `;
      
      notice.innerHTML = `
        <div style="display: flex; align-items: center; gap: 8px;">
          <span>🍎</span>
          <span>Kiro: Apple Music has limited API support - visual blocking only</span>
          <button id="kiro-dismiss-notice" style="background: none; border: none; color: white; cursor: pointer; margin-left: 8px;">✕</button>
        </div>
      `;
      
      document.body.appendChild(notice);
      
      // Auto-dismiss after 5 seconds or on click
      const dismissNotice = () => {
        notice.style.animation = 'fadeOut 0.3s ease-out';
        setTimeout(() => notice.remove(), 300);
      };
      
      document.getElementById('kiro-dismiss-notice').addEventListener('click', dismissNotice);
      setTimeout(dismissNotice, 5000);
    }

    extractArtistInfo(element) {
      // Try Apple Music-specific extraction first
      const appleMusicInfo = this.extractAppleMusicSpecific(element);
      if (appleMusicInfo) return appleMusicInfo;
      
      // Fall back to base implementation
      return super.extractArtistInfo(element);
    }

    extractAppleMusicSpecific(element) {
      // Extract from Apple Music href patterns
      if (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/')) {
        const artistId = element.href.split('/artist/')[1]?.split('/')[0];
        const artistName = element.textContent?.trim() || 
                          element.querySelector('.artist-name, .lockup__title')?.textContent?.trim();
        
        if (artistId && artistName) {
          return {
            name: artistName,
            appleMusicId: artistId,
            source: 'apple-music-href',
            element: element
          };
        }
      }

      // Extract from song row
      if (element.classList.contains('songs-list-row') || element.classList.contains('tracklist__item')) {
        const artistLink = element.querySelector('a[href*="/artist/"]');
        const artistText = element.querySelector('.songs-list-row__by-line, .tracklist__secondary-text');
        
        if (artistLink) {
          const artistId = artistLink.href.split('/artist/')[1]?.split('/')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (artistId && artistName) {
            return {
              name: artistName,
              appleMusicId: artistId,
              source: 'song-row',
              element: element,
              isTrackRow: true
            };
          }
        } else if (artistText) {
          // Fallback to text-based detection if no link
          const artistName = artistText.textContent?.trim();
          if (artistName) {
            return {
              name: artistName,
              source: 'song-row-text',
              element: element,
              isTrackRow: true
            };
          }
        }
      }

      // Extract from artist card/lockup
      if (element.classList.contains('grid-item--artist') || element.classList.contains('artist-lockup')) {
        const link = element.querySelector('a[href*="/artist/"]');
        const titleElement = element.querySelector('.lockup__title, .grid-item__title');
        
        if (link && titleElement) {
          const artistId = link.href.split('/artist/')[1]?.split('/')[0];
          const artistName = titleElement.textContent?.trim();
          
          if (artistId && artistName) {
            return {
              name: artistName,
              appleMusicId: artistId,
              source: 'artist-card',
              element: element
            };
          }
        }
      }

      // Extract from now playing area
      if (element.closest('.web-chrome-playback-controls') || element.closest('.playback-controls')) {
        const artistLink = element.querySelector('a[href*="/artist/"]') || 
                          (element.tagName.toLowerCase() === 'a' && element.href.includes('/artist/') ? element : null);
        
        if (artistLink) {
          const artistId = artistLink.href.split('/artist/')[1]?.split('/')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (artistId && artistName) {
            return {
              name: artistName,
              appleMusicId: artistId,
              source: 'now-playing',
              element: element,
              isNowPlaying: true
            };
          }
        }
      }

      // Extract from artist header
      if (element.closest('.artist-header')) {
        const artistName = element.querySelector('.artist-header__title, h1')?.textContent?.trim();
        if (artistName) {
          return {
            name: artistName,
            source: 'artist-header',
            element: element,
            isArtistPage: true
          };
        }
      }

      // Extract from album header with artist info
      if (element.closest('.album-header')) {
        const artistLink = element.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          const artistId = artistLink.href.split('/artist/')[1]?.split('/')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (artistId && artistName) {
            return {
              name: artistName,
              appleMusicId: artistId,
              source: 'album-header',
              element: element
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
      } else if (artistInfo.isArtistPage) {
        this.handleArtistPageBlock(element, artistInfo);
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
      const playButton = element.querySelector('.songs-list-row__play-button, .play-button');
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
        backdrop-filter: blur(5px);
      `;
      
      indicator.textContent = '🚫 Blocked';
      indicator.title = `Blocked artist: ${artistInfo.name}`;
      
      // Add click handler for quick actions
      indicator.addEventListener('click', (e) => {
        e.stopPropagation();
        this.showAppleMusicQuickActions(element, artistInfo, indicator);
      });
      
      element.appendChild(indicator);
    }

    showAppleMusicQuickActions(element, artistInfo, indicator) {
      const menu = document.createElement('div');
      menu.style.cssText = `
        position: absolute;
        right: 0;
        top: 100%;
        background: rgba(28, 28, 30, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 8px 0;
        min-width: 180px;
        z-index: 1000;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(20px);
      `;
      
      menu.innerHTML = `
        <div class="apple-menu-item" data-action="show-once">
          <span>👁️</span> Show Once
        </div>
        <div class="apple-menu-item" data-action="remove-from-dnp">
          <span>✅</span> Unblock Artist
        </div>
        <div class="apple-menu-item" data-action="add-to-dnp">
          <span>🚫</span> Block Permanently
        </div>
        <div class="apple-menu-divider"></div>
        <div class="apple-menu-item apple-menu-info">
          <span>ℹ️</span> Manual removal required
        </div>
      `;
      
      // Add Apple Music specific menu styles
      const style = document.createElement('style');
      style.textContent = `
        .apple-menu-item {
          padding: 10px 16px;
          color: rgba(255, 255, 255, 0.8);
          cursor: pointer;
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 13px;
          font-weight: 400;
          transition: all 0.2s ease;
        }
        .apple-menu-item:hover {
          background: rgba(255, 255, 255, 0.1);
          color: white;
        }
        .apple-menu-item.apple-menu-info {
          cursor: default;
          opacity: 0.6;
          font-size: 11px;
        }
        .apple-menu-item.apple-menu-info:hover {
          background: none;
        }
        .apple-menu-divider {
          height: 1px;
          background: rgba(255, 255, 255, 0.1);
          margin: 4px 0;
        }
      `;
      document.head.appendChild(style);
      
      // Add event listeners
      menu.addEventListener('click', async (e) => {
        const action = e.target.closest('.apple-menu-item')?.getAttribute('data-action');
        if (action) {
          await this.handleAppleMusicQuickAction(action, element, artistInfo);
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

    async handleAppleMusicQuickAction(action, element, artistInfo) {
      switch (action) {
        case 'show-once':
          this.showOnce(element, artistInfo);
          break;
        case 'remove-from-dnp':
          await this.removeFromDNP(element, artistInfo);
          break;
        case 'add-to-dnp':
          await this.addToDNP(artistInfo);
          break;
      }
    }

    showOnce(element, artistInfo) {
      // Temporarily unhide the element
      element.style.cssText = '';
      this.blockedElements.delete(element);
      
      // Remove indicator
      const indicator = element.querySelector('[title*="Blocked artist"]');
      if (indicator) {
        indicator.remove();
      }
      
      this.logAction('show_once', { artistInfo });
      
      // Show notification about manual action needed
      this.showAppleMusicNotification(
        `Showing ${artistInfo.name} - Manual removal required from Apple Music app`,
        'info'
      );
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
        
        // Show confirmation with Apple Music specific message
        this.showAppleMusicNotification(
          `Added ${artistInfo.name} to blocklist. Remove manually from Apple Music app.`,
          'success'
        );
      } catch (error) {
        console.error('Failed to add to DNP:', error);
        this.showAppleMusicNotification('Failed to add to blocklist', 'error');
      }
    }

    handleNowPlayingBlock(element, artistInfo) {
      // For Apple Music, we can't automatically skip, so show a prominent notification
      this.showSkipNotification(artistInfo, { 
        action: 'detected',
        message: 'Blocked artist detected - manual skip required'
      });
    }

    handleArtistPageBlock(element, artistInfo) {
      // Show a full-page overlay for blocked artist pages
      this.showArtistPageOverlay(artistInfo);
    }

    showArtistPageOverlay(artistInfo) {
      const overlay = document.createElement('div');
      overlay.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.95);
        color: white;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        z-index: 2147483647;
        backdrop-filter: blur(20px);
      `;
      
      overlay.innerHTML = `
        <div style="text-align: center; max-width: 400px; padding: 40px;">
          <div style="font-size: 48px; margin-bottom: 20px;">🚫</div>
          <h2 style="margin: 0 0 10px 0; font-size: 24px; font-weight: 600;">Artist Blocked</h2>
          <p style="margin: 0 0 20px 0; opacity: 0.8; font-size: 16px;">${artistInfo.name}</p>
          <p style="margin: 0 0 30px 0; opacity: 0.6; font-size: 14px; line-height: 1.4;">
            This artist is in your blocklist. Apple Music's limited API means you'll need to manually avoid their content.
          </p>
          <div style="display: flex; gap: 12px; justify-content: center;">
            <button id="kiro-unblock-artist" style="
              background: #007AFF;
              color: white;
              border: none;
              padding: 12px 24px;
              border-radius: 8px;
              font-size: 14px;
              font-weight: 500;
              cursor: pointer;
            ">Unblock Artist</button>
            <button id="kiro-go-back" style="
              background: rgba(255, 255, 255, 0.1);
              color: white;
              border: 1px solid rgba(255, 255, 255, 0.2);
              padding: 12px 24px;
              border-radius: 8px;
              font-size: 14px;
              font-weight: 500;
              cursor: pointer;
            ">Go Back</button>
          </div>
        </div>
      `;
      
      // Add event listeners
      overlay.querySelector('#kiro-unblock-artist').addEventListener('click', async () => {
        await this.removeFromDNP(null, artistInfo);
        overlay.remove();
      });
      
      overlay.querySelector('#kiro-go-back').addEventListener('click', () => {
        window.history.back();
        overlay.remove();
      });
      
      document.body.appendChild(overlay);
    }

    skipCurrentTrack(artistInfo) {
      // Apple Music doesn't allow programmatic skipping
      // Show notification instead
      this.showSkipNotification(artistInfo, {
        action: 'manual-skip-required',
        message: 'Please skip manually - Apple Music API limitations'
      });
    }

    showAppleMusicNotification(message, type = 'info') {
      const notification = document.createElement('div');
      const bgColor = type === 'success' ? '#34C759' : 
                     type === 'error' ? '#FF3B30' : 
                     '#007AFF';
      
      notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${bgColor};
        color: white;
        padding: 16px 20px;
        border-radius: 12px;
        font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
        font-size: 14px;
        font-weight: 500;
        z-index: 2147483647;
        animation: slideIn 0.3s ease-out;
        max-width: 350px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
        backdrop-filter: blur(20px);
      `;
      
      notification.innerHTML = `
        <div style="display: flex; align-items: flex-start; gap: 10px;">
          <span style="font-size: 16px;">🍎</span>
          <div>
            <div>${message}</div>
          </div>
        </div>
      `;
      
      document.body.appendChild(notification);
      
      // Remove after 5 seconds
      setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-in';
        setTimeout(() => notification.remove(), 300);
      }, 5000);
    }

    // Override to handle Apple Music's specific DOM structure
    async scanExistingContent() {
      // Scan common Apple Music containers
      const containers = [
        '.songs-list-row',
        '.tracklist__item',
        '.grid-item--artist',
        '.artist-lockup',
        '.grid-item--album',
        '.album-lockup',
        'a[href*="/artist/"]',
        '.search-results',
        '.shelf-grid',
        '.web-chrome-playback-controls',
        '.playback-controls',
        '.artist-header',
        '.album-header'
      ];

      for (const selector of containers) {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
          await this.processElement(element);
        }
      }
    }

    // Override to provide Apple Music specific track info
    extractTrackInfo(mediaElement) {
      // For Apple Music, we can get current track info from the playback controls
      const playbackControls = document.querySelector('.web-chrome-playback-controls, .playback-controls');
      if (playbackControls) {
        const artistLink = playbackControls.querySelector('a[href*="/artist/"]');
        if (artistLink) {
          return this.extractAppleMusicSpecific(artistLink);
        }
        
        // Fallback to text-based detection
        const artistText = playbackControls.querySelector('.playback-controls__artist, .web-chrome-playback-controls__artist');
        if (artistText) {
          return {
            name: artistText.textContent?.trim(),
            source: 'playback-controls-text',
            isNowPlaying: true
          };
        }
      }
      
      return super.extractTrackInfo(mediaElement);
    }

    // Add capability matrix information
    getCapabilities() {
      return {
        visualBlocking: true,
        autoSkip: false,
        libraryModification: false,
        playlistModification: false,
        limitations: [
          'Apple Music API has limited write capabilities',
          'Manual action required for content removal',
          'Auto-skip not supported',
          'Visual blocking and notifications only'
        ]
      };
    }
  }

  // Initialize Apple Music content script
  new AppleMusicContentScript();
};