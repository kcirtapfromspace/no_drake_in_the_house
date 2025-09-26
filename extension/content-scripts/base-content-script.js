/**
 * Base Content Script Framework
 * Provides common functionality for all streaming platform content scripts
 */

class BaseContentScript {
  constructor(platformConfig) {
    this.platform = platformConfig.platform;
    this.selectors = platformConfig.selectors;
    this.mediaSelectors = platformConfig.mediaSelectors;
    
    this.observer = null;
    this.shadowRoot = null;
    this.blockedElements = new WeakSet();
    this.processedElements = new WeakSet();
    this.telemetry = null;
    
    this.init();
  }

  async init() {
    try {
      // Load telemetry system
      await this.loadTelemetry();
      
      // Create isolated shadow DOM for UI components
      this.createShadowRoot();
      
      // Set up DOM monitoring
      this.setupMutationObserver();
      
      // Set up media event listeners
      this.setupMediaEventListeners();
      
      // Initial scan of existing content
      await this.scanExistingContent();
      
      console.log(`Kiro content script initialized for ${this.platform}`);
      
      if (this.telemetry) {
        this.telemetry.trackPerformance('initialization_complete', Date.now(), {
          platform: this.platform
        });
      }
    } catch (error) {
      console.error('Failed to initialize content script:', error);
      if (this.telemetry) {
        this.telemetry.trackError(error, { phase: 'initialization', platform: this.platform });
      }
    }
  }

  async loadTelemetry() {
    try {
      // Load telemetry script
      const script = document.createElement('script');
      script.src = chrome.runtime.getURL('utils/telemetry.js');
      document.head.appendChild(script);
      
      // Wait for script to load
      await new Promise((resolve) => {
        script.onload = resolve;
        setTimeout(resolve, 1000); // Fallback timeout
      });
      
      // Initialize telemetry
      if (window.TelemetryManager) {
        this.telemetry = new window.TelemetryManager();
      }
    } catch (error) {
      console.warn('Failed to load telemetry:', error);
    }
  }

  createShadowRoot() {
    // Create a container for our shadow DOM
    const container = document.createElement('div');
    container.id = 'kiro-extension-container';
    container.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      z-index: 2147483647;
      pointer-events: none;
    `;
    
    // Create shadow root for isolation
    this.shadowRoot = container.attachShadow({ mode: 'closed' });
    
    // Add styles for our UI components
    const styles = document.createElement('style');
    styles.textContent = `
      .kiro-badge {
        position: absolute;
        background: rgba(0, 0, 0, 0.8);
        color: white;
        padding: 2px 6px;
        border-radius: 3px;
        font-size: 11px;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        z-index: 1000;
        pointer-events: auto;
        cursor: pointer;
      }
      
      .kiro-hidden {
        opacity: 0.3;
        filter: grayscale(100%);
        position: relative;
      }
      
      .kiro-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 12px;
        z-index: 100;
        pointer-events: auto;
      }
      
      .kiro-controls {
        display: flex;
        gap: 8px;
        margin-top: 4px;
      }
      
      .kiro-btn {
        background: #1db954;
        color: white;
        border: none;
        padding: 4px 8px;
        border-radius: 3px;
        font-size: 10px;
        cursor: pointer;
      }
      
      .kiro-btn:hover {
        background: #1ed760;
      }
      
      .kiro-btn.secondary {
        background: #535353;
      }
      
      .kiro-btn.secondary:hover {
        background: #727272;
      }
    `;
    
    this.shadowRoot.appendChild(styles);
    
    // Add container to page
    document.documentElement.appendChild(container);
  }

  setupMutationObserver() {
    this.observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList') {
          this.processMutations(mutation.addedNodes);
        } else if (mutation.type === 'attributes') {
          this.processAttributeChange(mutation.target, mutation.attributeName);
        }
      }
    });

    this.observer.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['data-testid', 'aria-label', 'role', 'title', 'alt']
    });
  }

  setupMediaEventListeners() {
    // Listen for media events to implement auto-skip
    document.addEventListener('play', this.handleMediaPlay.bind(this), true);
    document.addEventListener('loadstart', this.handleMediaLoadStart.bind(this), true);
    document.addEventListener('loadedmetadata', this.handleMediaLoadedMetadata.bind(this), true);
    document.addEventListener('timeupdate', this.handleMediaTimeUpdate.bind(this), true);
    
    // Listen for custom media events from streaming platforms
    document.addEventListener('trackchange', this.handleTrackChange.bind(this), true);
    document.addEventListener('playerupdate', this.handlePlayerUpdate.bind(this), true);
    
    // Listen for navigation changes (SPA routing)
    let lastUrl = location.href;
    new MutationObserver(() => {
      const url = location.href;
      if (url !== lastUrl) {
        lastUrl = url;
        setTimeout(() => this.scanExistingContent(), 1000);
      }
    }).observe(document, { subtree: true, childList: true });
    
    // Set up periodic check for current track
    this.trackCheckInterval = setInterval(() => {
      this.checkCurrentlyPlayingTrack();
    }, 2000);
  }

  async processMutations(nodes) {
    for (const node of nodes) {
      if (node.nodeType === Node.ELEMENT_NODE) {
        await this.processElement(node);
        
        // Process child elements
        const childElements = node.querySelectorAll('*');
        for (const child of childElements) {
          await this.processElement(child);
        }
      }
    }
  }

  async processElement(element) {
    if (this.disabled || this.processedElements.has(element)) {
      return;
    }
    
    this.processedElements.add(element);
    
    const startTime = performance.now();
    const artistInfo = this.extractArtistInfo(element);
    const detectionTime = performance.now() - startTime;
    
    // Track artist detection attempt
    if (this.telemetry) {
      this.telemetry.trackArtistDetection(
        this.platform,
        element,
        artistInfo,
        detectionTime,
        !!artistInfo
      );
    }
    
    if (artistInfo) {
      const isBlocked = await this.checkIfBlocked(artistInfo);
      if (isBlocked) {
        this.hideElement(element, artistInfo);
        
        if (this.telemetry) {
          this.telemetry.trackUserAction('content_hidden', this.platform, artistInfo, {
            elementTag: element.tagName,
            source: artistInfo.source
          });
        }
      }
    }
  }

  processAttributeChange(element, attributeName) {
    // Re-process element if relevant attributes changed
    if (['data-testid', 'aria-label', 'title'].includes(attributeName)) {
      this.processedElements.delete(element);
      this.processElement(element);
    }
  }

  extractArtistInfo(element) {
    // This method should be overridden by platform-specific implementations
    // Default implementation tries common patterns with multiple strategies
    
    const strategies = [
      () => this.extractFromDataAttributes(element),
      () => this.extractFromAriaLabels(element),
      () => this.extractFromTextContent(element),
      () => this.extractFromLinks(element),
      () => this.extractFromImageAlt(element),
      () => this.extractFromMetadata(element),
      () => this.extractFromParentContext(element)
    ];

    for (const strategy of strategies) {
      const result = strategy();
      if (result && this.validateArtistInfo(result)) {
        return result;
      }
    }

    return null;
  }

  validateArtistInfo(artistInfo) {
    // Validate that we have meaningful artist information
    if (!artistInfo || !artistInfo.name) return false;
    
    const name = artistInfo.name.trim();
    
    // Filter out common false positives
    const falsePositives = [
      'play', 'pause', 'next', 'previous', 'shuffle', 'repeat',
      'like', 'dislike', 'share', 'download', 'add', 'remove',
      'playlist', 'album', 'track', 'song', 'artist', 'music',
      'loading', 'error', 'retry', 'cancel', 'ok', 'yes', 'no'
    ];
    
    if (falsePositives.includes(name.toLowerCase())) return false;
    if (name.length < 2 || name.length > 100) return false;
    if (/^\d+$/.test(name)) return false; // Just numbers
    if (/^[^a-zA-Z0-9\s]+$/.test(name)) return false; // Only special characters
    
    return true;
  }

  extractFromDataAttributes(element) {
    // Look for common data attributes
    const testId = element.getAttribute('data-testid');
    const uri = element.getAttribute('data-uri');
    
    if (testId && testId.includes('artist')) {
      const artistName = element.textContent?.trim();
      if (artistName) {
        return {
          name: artistName,
          source: 'data-testid',
          element: element
        };
      }
    }
    
    if (uri && uri.includes('artist')) {
      const artistId = uri.split(':').pop();
      const artistName = element.textContent?.trim();
      return {
        name: artistName,
        externalId: artistId,
        source: 'data-uri',
        element: element
      };
    }
    
    return null;
  }

  extractFromAriaLabels(element) {
    const ariaLabel = element.getAttribute('aria-label');
    if (ariaLabel) {
      // Look for patterns like "Artist: Name" or "by Artist Name"
      const patterns = [
        /Artist:\s*(.+)/i,
        /by\s+(.+)/i,
        /^(.+)\s+artist$/i
      ];
      
      for (const pattern of patterns) {
        const match = ariaLabel.match(pattern);
        if (match) {
          return {
            name: match[1].trim(),
            source: 'aria-label',
            element: element
          };
        }
      }
    }
    
    return null;
  }

  extractFromTextContent(element) {
    const text = element.textContent?.trim();
    if (!text || text.length > 100) return null;
    
    // Look for elements that might contain artist names
    const tagName = element.tagName.toLowerCase();
    const className = element.className || '';
    
    if (tagName === 'a' || className.includes('artist') || className.includes('creator')) {
      return {
        name: text,
        source: 'text-content',
        element: element
      };
    }
    
    return null;
  }

  extractFromLinks(element) {
    if (element.tagName.toLowerCase() === 'a') {
      const href = element.href;
      if (href && href.includes('/artist/')) {
        const artistId = href.split('/artist/')[1]?.split('/')[0];
        const artistName = element.textContent?.trim();
        
        if (artistId && artistName) {
          return {
            name: artistName,
            externalId: artistId,
            source: 'link-href',
            element: element
          };
        }
      }
    }
    
    return null;
  }

  extractFromImageAlt(element) {
    // Check for artist images with alt text
    const img = element.tagName.toLowerCase() === 'img' ? element : element.querySelector('img');
    if (img && img.alt) {
      const alt = img.alt.trim();
      
      // Look for patterns like "Artist Name", "Photo of Artist Name", etc.
      const patterns = [
        /^(.+)$/,
        /^Photo of (.+)$/i,
        /^Image of (.+)$/i,
        /^(.+) artist photo$/i,
        /^(.+) profile picture$/i
      ];
      
      for (const pattern of patterns) {
        const match = alt.match(pattern);
        if (match && match[1]) {
          const artistName = match[1].trim();
          if (artistName.length > 2 && artistName.length < 100) {
            return {
              name: artistName,
              source: 'image-alt',
              element: element
            };
          }
        }
      }
    }
    
    return null;
  }

  extractFromMetadata(element) {
    // Check for microdata or structured data
    const itemProp = element.getAttribute('itemprop');
    const itemType = element.getAttribute('itemtype');
    
    if (itemProp === 'byArtist' || itemProp === 'performer') {
      const artistName = element.textContent?.trim();
      if (artistName) {
        return {
          name: artistName,
          source: 'microdata',
          element: element
        };
      }
    }
    
    if (itemType && itemType.includes('MusicGroup')) {
      const nameElement = element.querySelector('[itemprop="name"]');
      if (nameElement) {
        return {
          name: nameElement.textContent?.trim(),
          source: 'microdata-group',
          element: element
        };
      }
    }
    
    return null;
  }

  extractFromParentContext(element) {
    // Look at parent elements for context clues
    let parent = element.parentElement;
    let depth = 0;
    
    while (parent && depth < 3) {
      // Check if parent has artist-related classes or attributes
      const className = parent.className || '';
      const dataTestId = parent.getAttribute('data-testid') || '';
      
      if (className.includes('artist') || dataTestId.includes('artist')) {
        const artistLink = parent.querySelector('a[href*="/artist/"]');
        if (artistLink && artistLink !== element) {
          const artistId = artistLink.href.split('/artist/')[1]?.split('/')[0];
          const artistName = artistLink.textContent?.trim();
          
          if (artistId && artistName) {
            return {
              name: artistName,
              externalId: artistId,
              source: 'parent-context',
              element: element,
              contextElement: parent
            };
          }
        }
      }
      
      parent = parent.parentElement;
      depth++;
    }
    
    return null;
  }

  async checkIfBlocked(artistInfo) {
    const startTime = performance.now();
    
    return new Promise((resolve) => {
      chrome.runtime.sendMessage({
        type: 'CHECK_ARTIST_BLOCKED',
        artistInfo: artistInfo
      }, (response) => {
        const lookupTime = performance.now() - startTime;
        const blocked = response?.blocked || false;
        
        // Track bloom filter performance
        if (this.telemetry) {
          this.telemetry.trackBloomFilterPerformance(blocked, lookupTime);
        }
        
        resolve(blocked);
      });
    });
  }

  hideElement(element, artistInfo) {
    if (this.blockedElements.has(element)) {
      return;
    }
    
    this.blockedElements.add(element);
    
    // Apply visual hiding
    element.classList.add('kiro-hidden');
    
    // Create overlay with controls
    this.createOverlay(element, artistInfo);
    
    // Log the action
    this.logAction('content_hidden', { artistInfo, element: element.tagName });
  }

  createOverlay(element, artistInfo) {
    const overlay = document.createElement('div');
    overlay.className = 'kiro-overlay';
    overlay.setAttribute('data-artist-id', artistInfo.externalId || artistInfo.name);
    
    const content = document.createElement('div');
    content.innerHTML = `
      <div class="kiro-overlay-header">
        <span class="kiro-overlay-icon">🚫</span>
        <span class="kiro-overlay-title">Hidden by Kiro</span>
      </div>
      <div class="kiro-overlay-artist">${artistInfo.name}</div>
      <div class="kiro-controls">
        <button class="kiro-btn primary" data-action="play-once" title="Allow this content once">
          <span>▶️</span> Play Once
        </button>
        <button class="kiro-btn secondary" data-action="remove-from-dnp" title="Remove from blocklist">
          <span>✅</span> Unblock
        </button>
        <button class="kiro-btn tertiary" data-action="add-to-dnp" title="Add to permanent blocklist">
          <span>🚫</span> Block
        </button>
      </div>
    `;
    
    // Add enhanced styles
    const overlayStyles = document.createElement('style');
    overlayStyles.textContent = `
      .kiro-overlay-header {
        display: flex;
        align-items: center;
        gap: 6px;
        margin-bottom: 4px;
        font-weight: 600;
        font-size: 12px;
      }
      
      .kiro-overlay-icon {
        font-size: 14px;
      }
      
      .kiro-overlay-artist {
        font-size: 11px;
        opacity: 0.8;
        margin-bottom: 8px;
        max-width: 200px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      
      .kiro-controls {
        display: flex;
        gap: 4px;
        flex-wrap: wrap;
        justify-content: center;
      }
      
      .kiro-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        border: none;
        border-radius: 4px;
        font-size: 10px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.2s;
        white-space: nowrap;
      }
      
      .kiro-btn:hover {
        transform: translateY(-1px);
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
      }
      
      .kiro-btn.primary {
        background: #1db954;
        color: white;
      }
      
      .kiro-btn.primary:hover {
        background: #1ed760;
      }
      
      .kiro-btn.secondary {
        background: #444;
        color: white;
      }
      
      .kiro-btn.secondary:hover {
        background: #555;
      }
      
      .kiro-btn.tertiary {
        background: rgba(255, 255, 255, 0.1);
        color: white;
        border: 1px solid rgba(255, 255, 255, 0.2);
      }
      
      .kiro-btn.tertiary:hover {
        background: rgba(255, 255, 255, 0.2);
      }
    `;
    
    content.appendChild(overlayStyles);
    
    // Add event listeners with better error handling
    content.addEventListener('click', async (e) => {
      const button = e.target.closest('[data-action]');
      if (!button) return;
      
      const action = button.getAttribute('data-action');
      const originalText = button.innerHTML;
      
      // Show loading state
      button.innerHTML = '<span>⏳</span> ...';
      button.disabled = true;
      
      try {
        switch (action) {
          case 'play-once':
            await this.playOnce(element, artistInfo);
            break;
          case 'remove-from-dnp':
            await this.removeFromDNP(element, artistInfo);
            break;
          case 'add-to-dnp':
            await this.addToDNP(artistInfo);
            break;
        }
      } catch (error) {
        console.error('Action failed:', error);
        // Restore button state on error
        button.innerHTML = originalText;
        button.disabled = false;
      }
    });
    
    overlay.appendChild(content);
    
    // Position overlay with better positioning logic
    this.positionOverlay(overlay, element);
    
    // Store reference for cleanup
    overlay._targetElement = element;
    overlay._artistInfo = artistInfo;
    
    this.shadowRoot.appendChild(overlay);
    
    // Update position on scroll/resize
    this.setupOverlayPositionUpdates(overlay, element);
    
    return overlay;
  }

  positionOverlay(overlay, element) {
    const rect = element.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    
    // Calculate optimal position
    let top = rect.top;
    let left = rect.left;
    let width = rect.width;
    let height = rect.height;
    
    // Ensure overlay doesn't go off-screen
    if (left + width > viewportWidth) {
      left = viewportWidth - width - 10;
    }
    if (top + height > viewportHeight) {
      top = viewportHeight - height - 10;
    }
    
    // Minimum size constraints
    width = Math.max(width, 200);
    height = Math.max(height, 80);
    
    overlay.style.cssText = `
      position: fixed;
      top: ${Math.max(0, top)}px;
      left: ${Math.max(0, left)}px;
      width: ${width}px;
      height: ${height}px;
      z-index: 2147483646;
      pointer-events: auto;
    `;
  }

  setupOverlayPositionUpdates(overlay, element) {
    // Update overlay position on scroll and resize
    const updatePosition = () => {
      if (overlay.parentNode && element.offsetParent) {
        this.positionOverlay(overlay, element);
      } else if (overlay.parentNode) {
        // Element is no longer visible, remove overlay
        overlay.remove();
      }
    };
    
    window.addEventListener('scroll', updatePosition, { passive: true });
    window.addEventListener('resize', updatePosition, { passive: true });
    
    // Store cleanup function
    overlay._cleanup = () => {
      window.removeEventListener('scroll', updatePosition);
      window.removeEventListener('resize', updatePosition);
    };
  }

  playOnce(element, artistInfo) {
    // Temporarily unhide the element
    element.classList.remove('kiro-hidden');
    this.blockedElements.delete(element);
    
    // Remove overlay
    const overlays = this.shadowRoot.querySelectorAll('.kiro-overlay');
    overlays.forEach(overlay => overlay.remove());
    
    this.logAction('play_once', { artistInfo });
  }

  async removeFromDNP(element, artistInfo) {
    try {
      await new Promise((resolve) => {
        chrome.runtime.sendMessage({
          type: 'REMOVE_FROM_DNP',
          artistInfo: artistInfo
        }, resolve);
      });
      
      // Unhide element
      element.classList.remove('kiro-hidden');
      this.blockedElements.delete(element);
      
      // Remove overlay
      const overlays = this.shadowRoot.querySelectorAll('.kiro-overlay');
      overlays.forEach(overlay => overlay.remove());
      
      this.logAction('removed_from_dnp', { artistInfo });
    } catch (error) {
      console.error('Failed to remove from DNP:', error);
    }
  }

  async handleMediaPlay(event) {
    const mediaElement = event.target;
    if (mediaElement.tagName.toLowerCase() === 'audio') {
      const trackInfo = this.extractTrackInfo(mediaElement);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipTrack(mediaElement, trackInfo);
        }
      }
    }
  }

  async handleMediaLoadStart(event) {
    // Similar to handleMediaPlay but for when media starts loading
    const mediaElement = event.target;
    if (mediaElement.tagName.toLowerCase() === 'audio') {
      const trackInfo = this.extractTrackInfo(mediaElement);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipTrack(mediaElement, trackInfo);
        }
      }
    }
  }

  async handleMediaLoadedMetadata(event) {
    // Check track when metadata is loaded
    const mediaElement = event.target;
    if (mediaElement.tagName.toLowerCase() === 'audio') {
      const trackInfo = this.extractTrackInfo(mediaElement);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipTrack(mediaElement, trackInfo);
        }
      }
    }
  }

  async handleMediaTimeUpdate(event) {
    // Check if we need to skip during playback (for tracks that start playing before we can detect them)
    const mediaElement = event.target;
    if (mediaElement.tagName.toLowerCase() === 'audio' && mediaElement.currentTime > 0 && mediaElement.currentTime < 2) {
      const trackInfo = this.extractTrackInfo(mediaElement);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipTrack(mediaElement, trackInfo);
        }
      }
    }
  }

  async handleTrackChange(event) {
    // Handle custom track change events from streaming platforms
    if (event.detail && event.detail.track) {
      const trackInfo = this.parseTrackData(event.detail.track);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipCurrentTrack(trackInfo);
        }
      }
    }
  }

  async handlePlayerUpdate(event) {
    // Handle custom player update events
    if (event.detail && event.detail.currentTrack) {
      const trackInfo = this.parseTrackData(event.detail.currentTrack);
      if (trackInfo) {
        const isBlocked = await this.checkIfBlocked(trackInfo);
        if (isBlocked) {
          this.skipCurrentTrack(trackInfo);
        }
      }
    }
  }

  parseTrackData(trackData) {
    // Parse track data from streaming platform events
    if (!trackData) return null;
    
    return {
      name: trackData.artist || trackData.artistName,
      trackTitle: trackData.title || trackData.name,
      externalId: trackData.artistId,
      source: 'platform-event'
    };
  }

  async checkCurrentlyPlayingTrack() {
    // Periodically check what's currently playing
    const nowPlayingInfo = this.getCurrentTrackInfo();
    if (nowPlayingInfo) {
      const isBlocked = await this.checkIfBlocked(nowPlayingInfo);
      if (isBlocked) {
        this.skipCurrentTrack(nowPlayingInfo);
      }
    }
  }

  getCurrentTrackInfo() {
    // This should be overridden by platform-specific implementations
    // Default implementation looks for common now-playing indicators
    
    const nowPlayingSelectors = [
      '[data-testid="now-playing"]',
      '.now-playing',
      '.current-track',
      '.player-track-info',
      '.playback-bar'
    ];
    
    for (const selector of nowPlayingSelectors) {
      const element = document.querySelector(selector);
      if (element) {
        const artistInfo = this.extractArtistInfo(element);
        if (artistInfo) {
          return { ...artistInfo, isNowPlaying: true };
        }
      }
    }
    
    return null;
  }

  skipCurrentTrack(trackInfo) {
    // Find and click the next button
    const nextSelectors = [
      '[data-testid="control-button-skip-forward"]',
      '[data-testid="next-button"]',
      '.next-button',
      '.skip-forward',
      '[aria-label*="Next"]',
      '[title*="Next"]'
    ];
    
    for (const selector of nextSelectors) {
      const nextButton = document.querySelector(selector);
      if (nextButton && !nextButton.disabled && nextButton.offsetParent !== null) {
        nextButton.click();
        this.showSkipNotification(trackInfo);
        this.logAction('track_auto_skipped', { trackInfo });
        return true;
      }
    }
    
    // If no next button found, try pausing
    const pauseSelectors = [
      '[data-testid="control-button-playpause"]',
      '.play-pause-button',
      '[aria-label*="Pause"]'
    ];
    
    for (const selector of pauseSelectors) {
      const pauseButton = document.querySelector(selector);
      if (pauseButton && pauseButton.offsetParent !== null) {
        pauseButton.click();
        this.showSkipNotification(trackInfo, { action: 'paused' });
        this.logAction('track_paused', { trackInfo });
        return true;
      }
    }
    
    return false;
  }

  extractTrackInfo(mediaElement) {
    // This should be overridden by platform-specific implementations
    // Default implementation looks for nearby elements
    
    const container = mediaElement.closest('[data-testid*="track"], [data-testid*="song"]');
    if (container) {
      return this.extractArtistInfo(container);
    }
    
    return null;
  }

  skipTrack(mediaElement, trackInfo) {
    // Pause and skip to next track
    mediaElement.pause();
    
    // Try to find and click next button
    const nextButton = document.querySelector(this.mediaSelectors?.nextButton || '[data-testid="control-button-skip-forward"]');
    if (nextButton) {
      nextButton.click();
    }
    
    // Show skip notification
    this.showSkipNotification(trackInfo);
    
    this.logAction('track_skipped', { trackInfo });
  }

  showSkipNotification(trackInfo, options = {}) {
    const notification = document.createElement('div');
    notification.className = 'kiro-skip-notification';
    
    const action = options.action || 'skipped';
    const icon = action === 'paused' ? '⏸️' : '⏭️';
    const actionText = action === 'paused' ? 'Paused' : 'Skipped';
    
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: rgba(0, 0, 0, 0.95);
      color: white;
      padding: 12px 16px;
      border-radius: 8px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
      z-index: 2147483647;
      animation: kiroSlideIn 0.3s ease-out;
      max-width: 300px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      border-left: 4px solid #ff4444;
      backdrop-filter: blur(10px);
    `;
    
    notification.innerHTML = `
      <div class="kiro-notification-header">
        <span class="kiro-notification-icon">${icon}</span>
        <span class="kiro-notification-title">${actionText} by Kiro</span>
      </div>
      <div class="kiro-notification-artist">${trackInfo.name}</div>
      ${trackInfo.trackTitle ? `<div class="kiro-notification-track">"${trackInfo.trackTitle}"</div>` : ''}
      <div class="kiro-notification-actions">
        <button class="kiro-notification-btn" data-action="undo">Undo</button>
        <button class="kiro-notification-btn" data-action="unblock">Unblock Artist</button>
      </div>
    `;
    
    // Add notification styles if not already present
    if (!document.getElementById('kiro-notification-styles')) {
      const style = document.createElement('style');
      style.id = 'kiro-notification-styles';
      style.textContent = `
        @keyframes kiroSlideIn {
          from { transform: translateX(100%); opacity: 0; }
          to { transform: translateX(0); opacity: 1; }
        }
        
        @keyframes kiroSlideOut {
          from { transform: translateX(0); opacity: 1; }
          to { transform: translateX(100%); opacity: 0; }
        }
        
        .kiro-notification-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 6px;
          font-weight: 600;
        }
        
        .kiro-notification-icon {
          font-size: 16px;
        }
        
        .kiro-notification-title {
          font-size: 13px;
        }
        
        .kiro-notification-artist {
          font-size: 12px;
          font-weight: 500;
          margin-bottom: 2px;
        }
        
        .kiro-notification-track {
          font-size: 11px;
          opacity: 0.8;
          margin-bottom: 8px;
          font-style: italic;
        }
        
        .kiro-notification-actions {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }
        
        .kiro-notification-btn {
          background: rgba(255, 255, 255, 0.1);
          color: white;
          border: 1px solid rgba(255, 255, 255, 0.2);
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 10px;
          cursor: pointer;
          transition: background-color 0.2s;
        }
        
        .kiro-notification-btn:hover {
          background: rgba(255, 255, 255, 0.2);
        }
      `;
      document.head.appendChild(style);
    }
    
    // Add event listeners for notification actions
    notification.addEventListener('click', async (e) => {
      const button = e.target.closest('[data-action]');
      if (!button) return;
      
      const action = button.getAttribute('data-action');
      
      try {
        if (action === 'undo') {
          // Try to go back to previous track
          const prevButton = document.querySelector('[data-testid="control-button-skip-back"], .previous-button, [aria-label*="Previous"]');
          if (prevButton && !prevButton.disabled) {
            prevButton.click();
          }
        } else if (action === 'unblock') {
          await this.removeFromDNP(null, trackInfo);
        }
        
        // Close notification
        notification.style.animation = 'kiroSlideOut 0.3s ease-in';
        setTimeout(() => notification.remove(), 300);
        
      } catch (error) {
        console.error('Notification action failed:', error);
      }
    });
    
    this.shadowRoot.appendChild(notification);
    
    // Auto-remove after duration
    const duration = options.duration || 5000;
    setTimeout(() => {
      if (notification.parentNode) {
        notification.style.animation = 'kiroSlideOut 0.3s ease-in';
        setTimeout(() => {
          if (notification.parentNode) {
            notification.remove();
          }
        }, 300);
      }
    }, duration);
    
    return notification;
  }

  async scanExistingContent() {
    // Scan all existing elements on the page
    const elements = document.querySelectorAll('*');
    for (const element of elements) {
      await this.processElement(element);
    }
  }

  logAction(action, data) {
    chrome.runtime.sendMessage({
      type: 'LOG_ACTION',
      action: {
        type: action,
        platform: this.platform,
        data: data,
        timestamp: Date.now()
      }
    });
  }

  destroy() {
    // Disconnect mutation observer
    if (this.observer) {
      this.observer.disconnect();
    }
    
    // Clear intervals
    if (this.trackCheckInterval) {
      clearInterval(this.trackCheckInterval);
    }
    
    // Clean up overlays
    if (this.shadowRoot) {
      const overlays = this.shadowRoot.querySelectorAll('.kiro-overlay');
      overlays.forEach(overlay => {
        if (overlay._cleanup) {
          overlay._cleanup();
        }
      });
    }
    
    // Remove shadow root container
    const container = document.getElementById('kiro-extension-container');
    if (container) {
      container.remove();
    }
    
    // Clear references
    this.blockedElements.clear();
    this.processedElements.clear();
    
    console.log(`Kiro content script destroyed for ${this.platform}`);
  }

  // Add method to temporarily disable extension
  disable() {
    this.disabled = true;
    
    // Remove all overlays
    if (this.shadowRoot) {
      const overlays = this.shadowRoot.querySelectorAll('.kiro-overlay');
      overlays.forEach(overlay => overlay.remove());
    }
    
    // Restore hidden elements
    this.blockedElements.forEach(element => {
      if (element && element.classList) {
        element.classList.remove('kiro-hidden');
      }
    });
    
    console.log(`Kiro content script disabled for ${this.platform}`);
  }

  // Add method to re-enable extension
  enable() {
    this.disabled = false;
    
    // Re-scan content
    setTimeout(() => this.scanExistingContent(), 500);
    
    console.log(`Kiro content script enabled for ${this.platform}`);
  }

  // Add method to get extension statistics
  getStats() {
    return {
      platform: this.platform,
      blockedElementsCount: this.blockedElements.size,
      processedElementsCount: this.processedElements.size,
      overlaysCount: this.shadowRoot ? this.shadowRoot.querySelectorAll('.kiro-overlay').length : 0
    };
  }
}

// Export for use by platform-specific scripts
window.BaseContentScript = BaseContentScript;