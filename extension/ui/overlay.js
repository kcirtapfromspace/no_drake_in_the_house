/**
 * Overlay UI Components for NDITH Extension
 * Provides reusable UI components for content scripts
 */

class NDITHOverlayUI {
  constructor() {
    this.overlays = new Map();
    this.notifications = [];
  }

  createBlockedContentOverlay(element, artistInfo, options = {}) {
    const overlayId = `ndith-overlay-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    const overlay = document.createElement('div');
    overlay.id = overlayId;
    overlay.className = 'ndith-content-overlay';
    
    // Position overlay over the blocked element
    const rect = element.getBoundingClientRect();
    overlay.style.cssText = `
      position: fixed;
      top: ${rect.top}px;
      left: ${rect.left}px;
      width: ${rect.width}px;
      height: ${rect.height}px;
      background: rgba(0, 0, 0, 0.8);
      color: white;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      z-index: 2147483646;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 12px;
      border-radius: 4px;
      backdrop-filter: blur(2px);
      pointer-events: auto;
      cursor: pointer;
    `;
    
    overlay.innerHTML = `
      <div class="ndith-overlay-content">
        <div class="ndith-overlay-icon">🚫</div>
        <div class="ndith-overlay-title">Hidden by NDITH</div>
        <div class="ndith-overlay-subtitle">${artistInfo.name}</div>
        <div class="ndith-overlay-actions">
          <button class="ndith-overlay-btn primary" data-action="play-once">
            Play Once
          </button>
          <button class="ndith-overlay-btn secondary" data-action="unblock">
            Unblock
          </button>
        </div>
      </div>
    `;
    
    // Add styles
    const style = document.createElement('style');
    style.textContent = `
      .ndith-overlay-content {
        text-align: center;
        padding: 16px;
      }
      
      .ndith-overlay-icon {
        font-size: 24px;
        margin-bottom: 8px;
      }
      
      .ndith-overlay-title {
        font-weight: 600;
        margin-bottom: 4px;
        font-size: 13px;
      }
      
      .ndith-overlay-subtitle {
        opacity: 0.8;
        margin-bottom: 12px;
        font-size: 11px;
      }
      
      .ndith-overlay-actions {
        display: flex;
        gap: 8px;
        justify-content: center;
      }
      
      .ndith-overlay-btn {
        padding: 6px 12px;
        border: none;
        border-radius: 4px;
        font-size: 11px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.2s;
      }
      
      .ndith-overlay-btn.primary {
        background: #1db954;
        color: white;
      }
      
      .ndith-overlay-btn.primary:hover {
        background: #1ed760;
        transform: translateY(-1px);
      }
      
      .ndith-overlay-btn.secondary {
        background: rgba(255, 255, 255, 0.1);
        color: white;
        border: 1px solid rgba(255, 255, 255, 0.2);
      }
      
      .ndith-overlay-btn.secondary:hover {
        background: rgba(255, 255, 255, 0.2);
        transform: translateY(-1px);
      }
    `;
    
    overlay.appendChild(style);
    
    // Add event listeners
    overlay.addEventListener('click', (e) => {
      const action = e.target.getAttribute('data-action');
      if (action && options.onAction) {
        options.onAction(action, artistInfo, element);
      }
    });
    
    // Store overlay reference
    this.overlays.set(overlayId, {
      element: overlay,
      targetElement: element,
      artistInfo: artistInfo
    });
    
    return overlay;
  }

  createSkipNotification(artistInfo, options = {}) {
    const notification = document.createElement('div');
    notification.className = 'ndith-skip-notification';
    
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: rgba(0, 0, 0, 0.9);
      color: white;
      padding: 12px 16px;
      border-radius: 8px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
      z-index: 2147483647;
      animation: ndithSlideIn 0.3s ease-out;
      max-width: 300px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      border-left: 4px solid #ff4444;
    `;
    
    notification.innerHTML = `
      <div class="ndith-notification-content">
        <div class="ndith-notification-title">
          <span class="ndith-notification-icon">⏭️</span>
          Track Skipped
        </div>
        <div class="ndith-notification-subtitle">
          ${artistInfo.name} is blocked
        </div>
      </div>
    `;
    
    // Add animation styles if not already present
    if (!document.getElementById('ndith-animations')) {
      const animationStyle = document.createElement('style');
      animationStyle.id = 'ndith-animations';
      animationStyle.textContent = `
        @keyframes ndithSlideIn {
          from { 
            transform: translateX(100%); 
            opacity: 0; 
          }
          to { 
            transform: translateX(0); 
            opacity: 1; 
          }
        }
        
        @keyframes ndithSlideOut {
          from { 
            transform: translateX(0); 
            opacity: 1; 
          }
          to { 
            transform: translateX(100%); 
            opacity: 0; 
          }
        }
        
        .ndith-notification-content {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }
        
        .ndith-notification-title {
          display: flex;
          align-items: center;
          gap: 8px;
          font-weight: 600;
          font-size: 13px;
        }
        
        .ndith-notification-subtitle {
          font-size: 12px;
          opacity: 0.8;
          margin-left: 24px;
        }
        
        .ndith-notification-icon {
          font-size: 16px;
        }
      `;
      document.head.appendChild(animationStyle);
    }
    
    // Auto-remove after duration
    const duration = options.duration || 3000;
    setTimeout(() => {
      notification.style.animation = 'ndithSlideOut 0.3s ease-in';
      setTimeout(() => {
        if (notification.parentNode) {
          notification.remove();
        }
      }, 300);
    }, duration);
    
    this.notifications.push(notification);
    return notification;
  }

  createContextMenu(element, artistInfo, actions = []) {
    const menu = document.createElement('div');
    menu.className = 'ndith-context-menu';
    
    const rect = element.getBoundingClientRect();
    menu.style.cssText = `
      position: fixed;
      top: ${rect.bottom + 5}px;
      left: ${rect.left}px;
      background: #2a2a2a;
      border: 1px solid #404040;
      border-radius: 6px;
      padding: 4px 0;
      min-width: 160px;
      z-index: 2147483647;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 13px;
    `;
    
    // Default actions if none provided
    if (actions.length === 0) {
      actions = [
        { id: 'play-once', label: '▶️ Play Once', type: 'primary' },
        { id: 'unblock', label: '✅ Unblock Artist', type: 'default' },
        { id: 'block-permanent', label: '🚫 Block Permanently', type: 'danger' }
      ];
    }
    
    actions.forEach(action => {
      const item = document.createElement('div');
      item.className = `ndith-menu-item ${action.type || 'default'}`;
      item.setAttribute('data-action', action.id);
      item.textContent = action.label;
      
      item.style.cssText = `
        padding: 8px 16px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        color: #e0e0e0;
        transition: background-color 0.2s;
      `;
      
      item.addEventListener('mouseenter', () => {
        item.style.backgroundColor = action.type === 'danger' ? '#ff4444' : '#404040';
      });
      
      item.addEventListener('mouseleave', () => {
        item.style.backgroundColor = 'transparent';
      });
      
      menu.appendChild(item);
    });
    
    // Close menu when clicking outside
    const closeMenu = (e) => {
      if (!menu.contains(e.target)) {
        menu.remove();
        document.removeEventListener('click', closeMenu);
      }
    };
    
    setTimeout(() => {
      document.addEventListener('click', closeMenu);
    }, 100);
    
    return menu;
  }

  createBadge(text, options = {}) {
    const badge = document.createElement('div');
    badge.className = 'ndith-badge';
    badge.textContent = text;
    
    const type = options.type || 'default';
    const colors = {
      default: { bg: 'rgba(0, 0, 0, 0.8)', color: 'white' },
      success: { bg: '#1db954', color: 'white' },
      warning: { bg: '#ff9500', color: 'white' },
      danger: { bg: '#ff4444', color: 'white' }
    };
    
    const color = colors[type] || colors.default;
    
    badge.style.cssText = `
      position: absolute;
      top: ${options.top || '4px'};
      right: ${options.right || '4px'};
      background: ${color.bg};
      color: ${color.color};
      padding: 2px 6px;
      border-radius: 10px;
      font-size: 10px;
      font-weight: 600;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      z-index: 1000;
      pointer-events: auto;
      cursor: pointer;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    `;
    
    return badge;
  }

  removeOverlay(overlayId) {
    const overlay = this.overlays.get(overlayId);
    if (overlay) {
      overlay.element.remove();
      this.overlays.delete(overlayId);
    }
  }

  removeAllOverlays() {
    this.overlays.forEach((overlay, id) => {
      overlay.element.remove();
    });
    this.overlays.clear();
  }

  updateOverlayPositions() {
    // Update overlay positions when page layout changes
    this.overlays.forEach((overlay) => {
      const rect = overlay.targetElement.getBoundingClientRect();
      overlay.element.style.top = `${rect.top}px`;
      overlay.element.style.left = `${rect.left}px`;
      overlay.element.style.width = `${rect.width}px`;
      overlay.element.style.height = `${rect.height}px`;
    });
  }
}

// Export for use by content scripts
window.NDITHOverlayUI = NDITHOverlayUI;