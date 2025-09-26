/**
 * Signed Updates System for Kiro Extension
 * Handles cryptographically signed DNP filter updates for security
 */

class SignedUpdateManager {
  constructor() {
    this.publicKey = null;
    this.updateEndpoint = 'http://localhost:3000/api/v1/dnp/signed-update';
    this.maxUpdateAge = 24 * 60 * 60 * 1000; // 24 hours
  }

  async init() {
    try {
      // Load or generate public key for signature verification
      await this.loadPublicKey();
    } catch (error) {
      console.error('Failed to initialize signed updates:', error);
    }
  }

  async loadPublicKey() {
    try {
      // In a real implementation, this would be a hardcoded public key
      // or loaded from a secure source
      const result = await chrome.storage.local.get(['publicKey']);
      
      if (result.publicKey) {
        this.publicKey = await this.importPublicKey(result.publicKey);
      } else {
        // For demo purposes, we'll skip signature verification
        console.warn('No public key found - signature verification disabled');
      }
    } catch (error) {
      console.error('Failed to load public key:', error);
    }
  }

  async importPublicKey(keyData) {
    try {
      return await crypto.subtle.importKey(
        'spki',
        this.base64ToArrayBuffer(keyData),
        {
          name: 'RSA-PSS',
          hash: 'SHA-256'
        },
        false,
        ['verify']
      );
    } catch (error) {
      console.error('Failed to import public key:', error);
      return null;
    }
  }

  base64ToArrayBuffer(base64) {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }

  async verifySignature(data, signature) {
    if (!this.publicKey) {
      console.warn('No public key available - skipping signature verification');
      return true; // Allow in demo mode
    }

    try {
      const encoder = new TextEncoder();
      const dataBuffer = encoder.encode(JSON.stringify(data));
      const signatureBuffer = this.base64ToArrayBuffer(signature);

      const isValid = await crypto.subtle.verify(
        {
          name: 'RSA-PSS',
          saltLength: 32
        },
        this.publicKey,
        signatureBuffer,
        dataBuffer
      );

      return isValid;
    } catch (error) {
      console.error('Signature verification failed:', error);
      return false;
    }
  }

  async fetchSignedUpdate(authToken) {
    try {
      if (!navigator.onLine) {
        return null;
      }

      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 15000);

      const response = await fetch(this.updateEndpoint, {
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'Content-Type': 'application/json'
        },
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const signedUpdate = await response.json();
      
      // Validate update structure
      if (!this.validateUpdateStructure(signedUpdate)) {
        throw new Error('Invalid update structure');
      }

      // Verify signature
      const isValidSignature = await this.verifySignature(
        signedUpdate.data,
        signedUpdate.signature
      );

      if (!isValidSignature) {
        throw new Error('Invalid signature');
      }

      // Check update age
      const updateAge = Date.now() - signedUpdate.data.timestamp;
      if (updateAge > this.maxUpdateAge) {
        throw new Error('Update is too old');
      }

      return signedUpdate.data;
    } catch (error) {
      console.error('Failed to fetch signed update:', error);
      return null;
    }
  }

  validateUpdateStructure(signedUpdate) {
    return (
      signedUpdate &&
      typeof signedUpdate === 'object' &&
      signedUpdate.data &&
      typeof signedUpdate.data === 'object' &&
      signedUpdate.signature &&
      typeof signedUpdate.signature === 'string' &&
      signedUpdate.data.version &&
      signedUpdate.data.timestamp &&
      Array.isArray(signedUpdate.data.artists) &&
      signedUpdate.data.bloomFilter
    );
  }

  async applySignedUpdate(updateData, dnpFilterManager) {
    try {
      // Validate update data
      if (!updateData || !updateData.artists || !updateData.bloomFilter) {
        throw new Error('Invalid update data');
      }

      // Check if this is a newer version
      const currentVersion = await this.getCurrentVersion();
      if (updateData.version <= currentVersion) {
        console.log('Update is not newer than current version');
        return false;
      }

      // Apply the update
      dnpFilterManager.fullDNPList = updateData.artists;
      
      // Restore bloom filter
      if (updateData.bloomFilter) {
        try {
          dnpFilterManager.bloomFilter = BloomFilter.deserialize(updateData.bloomFilter);
        } catch (error) {
          console.warn('Failed to restore bloom filter, rebuilding:', error);
          await dnpFilterManager.rebuildFilter();
        }
      }

      // Save update metadata
      await chrome.storage.local.set({
        currentUpdateVersion: updateData.version,
        lastSignedUpdate: Date.now(),
        updateSource: 'signed'
      });

      // Save the updated data
      await dnpFilterManager.saveToStorage();

      console.log(`Applied signed update version ${updateData.version} with ${updateData.artists.length} artists`);
      return true;
    } catch (error) {
      console.error('Failed to apply signed update:', error);
      return false;
    }
  }

  async getCurrentVersion() {
    const result = await chrome.storage.local.get(['currentUpdateVersion']);
    return result.currentUpdateVersion || 0;
  }

  async getUpdateStatus() {
    const result = await chrome.storage.local.get([
      'currentUpdateVersion',
      'lastSignedUpdate',
      'updateSource'
    ]);

    const now = Date.now();
    const lastUpdate = result.lastSignedUpdate || 0;
    const updateAge = now - lastUpdate;

    return {
      currentVersion: result.currentUpdateVersion || 0,
      lastUpdate: lastUpdate,
      updateAge: updateAge,
      updateSource: result.updateSource || 'unknown',
      isStale: updateAge > this.maxUpdateAge,
      hasSignedUpdates: !!result.lastSignedUpdate
    };
  }

  // Create a hash of the current DNP list for integrity checking
  async createIntegrityHash(dnpList) {
    try {
      const encoder = new TextEncoder();
      const data = encoder.encode(JSON.stringify(dnpList.sort()));
      const hashBuffer = await crypto.subtle.digest('SHA-256', data);
      return this.arrayBufferToBase64(hashBuffer);
    } catch (error) {
      console.error('Failed to create integrity hash:', error);
      return null;
    }
  }

  async verifyIntegrity(dnpList, expectedHash) {
    if (!expectedHash) return true; // Skip if no hash provided
    
    const currentHash = await this.createIntegrityHash(dnpList);
    return currentHash === expectedHash;
  }

  // Generate update metadata for debugging
  generateUpdateMetadata(dnpList) {
    return {
      timestamp: Date.now(),
      artistCount: dnpList.length,
      platforms: this.getPlatformStats(dnpList),
      version: Date.now() // Simple versioning based on timestamp
    };
  }

  getPlatformStats(dnpList) {
    const stats = {
      spotify: 0,
      appleMusic: 0,
      youtubeMusic: 0,
      tidal: 0,
      unknown: 0
    };

    for (const artist of dnpList) {
      if (artist.spotifyId) stats.spotify++;
      if (artist.appleMusicId) stats.appleMusic++;
      if (artist.youtubeMusicId) stats.youtubeMusic++;
      if (artist.tidalId) stats.tidal++;
      if (!artist.spotifyId && !artist.appleMusicId && !artist.youtubeMusicId && !artist.tidalId) {
        stats.unknown++;
      }
    }

    return stats;
  }
}

// Export for use in extension
if (typeof module !== 'undefined' && module.exports) {
  module.exports = SignedUpdateManager;
} else {
  window.SignedUpdateManager = SignedUpdateManager;
}