/**
 * Telemetry System for Kiro Extension
 * Monitors performance, selector drift, and usage patterns
 */

class TelemetryManager {
  constructor() {
    this.sessionId = this.generateSessionId();
    this.startTime = Date.now();
    this.events = [];
    this.selectorStats = new Map();
    this.performanceMetrics = {
      artistDetections: 0,
      successfulDetections: 0,
      falsePositives: 0,
      averageDetectionTime: 0,
      bloomFilterHits: 0,
      bloomFilterMisses: 0
    };
    
    this.maxEvents = 1000; // Limit stored events
    this.reportInterval = 10 * 60 * 1000; // 10 minutes
    
    this.setupPeriodicReporting();
  }

  generateSessionId() {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  // Track artist detection attempts
  trackArtistDetection(platform, element, artistInfo, detectionTime, success) {
    const event = {
      type: 'artist_detection',
      timestamp: Date.now(),
      sessionId: this.sessionId,
      platform: platform,
      success: success,
      detectionTime: detectionTime,
      elementTag: element?.tagName?.toLowerCase(),
      elementClasses: element?.className,
      elementTestId: element?.getAttribute?.('data-testid'),
      artistInfo: artistInfo ? {
        name: artistInfo.name,
        source: artistInfo.source,
        hasExternalId: !!artistInfo.externalId
      } : null
    };
    
    this.addEvent(event);
    
    // Update performance metrics
    this.performanceMetrics.artistDetections++;
    if (success) {
      this.performanceMetrics.successfulDetections++;
    }
    
    // Update average detection time
    const totalTime = this.performanceMetrics.averageDetectionTime * (this.performanceMetrics.artistDetections - 1);
    this.performanceMetrics.averageDetectionTime = (totalTime + detectionTime) / this.performanceMetrics.artistDetections;
  }

  // Track selector usage and success rates
  trackSelectorUsage(platform, selector, success, element) {
    const key = `${platform}:${selector}`;
    
    if (!this.selectorStats.has(key)) {
      this.selectorStats.set(key, {
        platform: platform,
        selector: selector,
        attempts: 0,
        successes: 0,
        lastUsed: 0,
        elements: new Set()
      });
    }
    
    const stats = this.selectorStats.get(key);
    stats.attempts++;
    stats.lastUsed = Date.now();
    
    if (success) {
      stats.successes++;
    }
    
    // Track element diversity
    if (element) {
      const elementSignature = `${element.tagName}:${element.className}`;
      stats.elements.add(elementSignature);
    }
    
    // Log potential selector drift
    if (stats.attempts > 10 && (stats.successes / stats.attempts) < 0.1) {
      this.trackSelectorDrift(platform, selector, stats);
    }
  }

  // Track selector drift (when selectors stop working)
  trackSelectorDrift(platform, selector, stats) {
    const event = {
      type: 'selector_drift',
      timestamp: Date.now(),
      sessionId: this.sessionId,
      platform: platform,
      selector: selector,
      successRate: stats.successes / stats.attempts,
      attempts: stats.attempts,
      elementDiversity: stats.elements.size
    };
    
    this.addEvent(event);
    console.warn(`Potential selector drift detected: ${platform} - ${selector} (${(stats.successRate * 100).toFixed(1)}% success rate)`);
  }

  // Track bloom filter performance
  trackBloomFilterPerformance(hit, lookupTime) {
    if (hit) {
      this.performanceMetrics.bloomFilterHits++;
    } else {
      this.performanceMetrics.bloomFilterMisses++;
    }
    
    // Track slow lookups
    if (lookupTime > 10) { // More than 10ms is considered slow
      const event = {
        type: 'slow_bloom_lookup',
        timestamp: Date.now(),
        sessionId: this.sessionId,
        lookupTime: lookupTime,
        hit: hit
      };
      
      this.addEvent(event);
    }
  }

  // Track user actions
  trackUserAction(action, platform, artistInfo, context = {}) {
    const event = {
      type: 'user_action',
      timestamp: Date.now(),
      sessionId: this.sessionId,
      action: action,
      platform: platform,
      artistInfo: artistInfo ? {
        name: artistInfo.name,
        source: artistInfo.source
      } : null,
      context: context
    };
    
    this.addEvent(event);
  }

  // Track performance metrics
  trackPerformance(metric, value, context = {}) {
    const event = {
      type: 'performance',
      timestamp: Date.now(),
      sessionId: this.sessionId,
      metric: metric,
      value: value,
      context: context
    };
    
    this.addEvent(event);
  }

  // Track errors
  trackError(error, context = {}) {
    const event = {
      type: 'error',
      timestamp: Date.now(),
      sessionId: this.sessionId,
      error: {
        message: error.message,
        stack: error.stack,
        name: error.name
      },
      context: context
    };
    
    this.addEvent(event);
  }

  addEvent(event) {
    this.events.push(event);
    
    // Limit stored events
    if (this.events.length > this.maxEvents) {
      this.events = this.events.slice(-this.maxEvents);
    }
  }

  // Get current session statistics
  getSessionStats() {
    const sessionDuration = Date.now() - this.startTime;
    const selectorStatsArray = Array.from(this.selectorStats.values()).map(stats => ({
      ...stats,
      successRate: stats.attempts > 0 ? stats.successes / stats.attempts : 0,
      elements: stats.elements.size
    }));
    
    return {
      sessionId: this.sessionId,
      sessionDuration: sessionDuration,
      totalEvents: this.events.length,
      performanceMetrics: this.performanceMetrics,
      selectorStats: selectorStatsArray,
      eventTypes: this.getEventTypeCounts()
    };
  }

  getEventTypeCounts() {
    const counts = {};
    for (const event of this.events) {
      counts[event.type] = (counts[event.type] || 0) + 1;
    }
    return counts;
  }

  // Generate telemetry report
  generateReport() {
    const stats = this.getSessionStats();
    
    // Identify problematic selectors
    const problematicSelectors = stats.selectorStats
      .filter(s => s.attempts > 5 && s.successRate < 0.5)
      .sort((a, b) => a.successRate - b.successRate);
    
    // Get recent errors
    const recentErrors = this.events
      .filter(e => e.type === 'error' && Date.now() - e.timestamp < 60000)
      .slice(-10);
    
    return {
      ...stats,
      problematicSelectors: problematicSelectors,
      recentErrors: recentErrors,
      recommendations: this.generateRecommendations(stats)
    };
  }

  generateRecommendations(stats) {
    const recommendations = [];
    
    // Check detection success rate
    if (stats.performanceMetrics.artistDetections > 10) {
      const successRate = stats.performanceMetrics.successfulDetections / stats.performanceMetrics.artistDetections;
      if (successRate < 0.7) {
        recommendations.push({
          type: 'low_detection_rate',
          message: `Artist detection success rate is low (${(successRate * 100).toFixed(1)}%)`,
          suggestion: 'Consider updating selectors or detection strategies'
        });
      }
    }
    
    // Check bloom filter efficiency
    const totalBloomLookups = stats.performanceMetrics.bloomFilterHits + stats.performanceMetrics.bloomFilterMisses;
    if (totalBloomLookups > 100) {
      const hitRate = stats.performanceMetrics.bloomFilterHits / totalBloomLookups;
      if (hitRate > 0.8) {
        recommendations.push({
          type: 'high_bloom_hit_rate',
          message: `Bloom filter hit rate is high (${(hitRate * 100).toFixed(1)}%)`,
          suggestion: 'Consider rebuilding bloom filter to reduce false positives'
        });
      }
    }
    
    // Check for selector drift
    const driftingSelectors = stats.selectorStats.filter(s => s.attempts > 10 && s.successRate < 0.2);
    if (driftingSelectors.length > 0) {
      recommendations.push({
        type: 'selector_drift',
        message: `${driftingSelectors.length} selectors may be drifting`,
        suggestion: 'Update selectors for affected platforms',
        selectors: driftingSelectors.map(s => `${s.platform}:${s.selector}`)
      });
    }
    
    return recommendations;
  }

  // Send telemetry data (if user consents and server is available)
  async sendTelemetry() {
    try {
      // Check if telemetry is enabled
      const settings = await chrome.storage.sync.get(['telemetryEnabled', 'authToken']);
      if (!settings.telemetryEnabled) {
        return false;
      }
      
      const report = this.generateReport();
      
      // Only send if we have meaningful data
      if (report.totalEvents < 5) {
        return false;
      }
      
      // Send to server (if available)
      if (settings.authToken) {
        const response = await fetch('http://localhost:3000/api/v1/telemetry', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${settings.authToken}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(report)
        });
        
        if (response.ok) {
          console.log('Telemetry data sent successfully');
          this.clearOldEvents();
          return true;
        }
      }
      
      return false;
    } catch (error) {
      console.error('Failed to send telemetry:', error);
      return false;
    }
  }

  clearOldEvents() {
    // Keep only recent events after successful telemetry send
    const cutoff = Date.now() - (60 * 60 * 1000); // 1 hour
    this.events = this.events.filter(event => event.timestamp > cutoff);
  }

  setupPeriodicReporting() {
    // Send telemetry periodically
    setInterval(() => {
      this.sendTelemetry();
    }, this.reportInterval);
    
    // Send telemetry on page unload
    window.addEventListener('beforeunload', () => {
      this.sendTelemetry();
    });
  }

  // Export data for debugging
  exportData() {
    return {
      sessionStats: this.getSessionStats(),
      events: this.events.slice(-100), // Last 100 events
      report: this.generateReport()
    };
  }
}

// Export for use in extension
if (typeof module !== 'undefined' && module.exports) {
  module.exports = TelemetryManager;
} else {
  window.TelemetryManager = TelemetryManager;
}