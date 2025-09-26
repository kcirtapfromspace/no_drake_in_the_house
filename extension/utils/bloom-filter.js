/**
 * Bloom Filter Implementation for Kiro Extension
 * Provides O(1) lookups for DNP list checking with minimal memory usage
 */

class BloomFilter {
  constructor(expectedElements = 10000, falsePositiveRate = 0.01) {
    this.expectedElements = expectedElements;
    this.falsePositiveRate = falsePositiveRate;
    
    // Calculate optimal bit array size and number of hash functions
    this.bitArraySize = this.calculateOptimalBitArraySize(expectedElements, falsePositiveRate);
    this.numHashFunctions = this.calculateOptimalHashFunctions(this.bitArraySize, expectedElements);
    
    // Initialize bit array
    this.bitArray = new Uint8Array(Math.ceil(this.bitArraySize / 8));
    this.elementCount = 0;
    
    console.log(`BloomFilter initialized: ${this.bitArraySize} bits, ${this.numHashFunctions} hash functions`);
  }

  calculateOptimalBitArraySize(n, p) {
    // m = -(n * ln(p)) / (ln(2)^2)
    return Math.ceil(-(n * Math.log(p)) / (Math.log(2) * Math.log(2)));
  }

  calculateOptimalHashFunctions(m, n) {
    // k = (m/n) * ln(2)
    return Math.ceil((m / n) * Math.log(2));
  }

  // Simple hash function using FNV-1a algorithm
  hash1(str) {
    let hash = 2166136261;
    for (let i = 0; i < str.length; i++) {
      hash ^= str.charCodeAt(i);
      hash *= 16777619;
    }
    return Math.abs(hash) % this.bitArraySize;
  }

  // Second hash function using djb2 algorithm
  hash2(str) {
    let hash = 5381;
    for (let i = 0; i < str.length; i++) {
      hash = ((hash << 5) + hash) + str.charCodeAt(i);
    }
    return Math.abs(hash) % this.bitArraySize;
  }

  // Generate multiple hash values using double hashing
  getHashValues(str) {
    const h1 = this.hash1(str);
    const h2 = this.hash2(str);
    const hashes = [];
    
    for (let i = 0; i < this.numHashFunctions; i++) {
      hashes.push((h1 + i * h2) % this.bitArraySize);
    }
    
    return hashes;
  }

  setBit(index) {
    const byteIndex = Math.floor(index / 8);
    const bitIndex = index % 8;
    this.bitArray[byteIndex] |= (1 << bitIndex);
  }

  getBit(index) {
    const byteIndex = Math.floor(index / 8);
    const bitIndex = index % 8;
    return (this.bitArray[byteIndex] & (1 << bitIndex)) !== 0;
  }

  add(item) {
    const str = this.normalizeString(item);
    const hashes = this.getHashValues(str);
    
    for (const hash of hashes) {
      this.setBit(hash);
    }
    
    this.elementCount++;
  }

  contains(item) {
    const str = this.normalizeString(item);
    const hashes = this.getHashValues(str);
    
    for (const hash of hashes) {
      if (!this.getBit(hash)) {
        return false; // Definitely not in set
      }
    }
    
    return true; // Probably in set (may be false positive)
  }

  normalizeString(item) {
    if (typeof item === 'string') {
      return item.toLowerCase().trim();
    } else if (item && typeof item === 'object') {
      // Handle artist objects
      const keys = [
        item.name?.toLowerCase().trim(),
        item.spotifyId,
        item.appleMusicId,
        item.youtubeMusicId,
        item.tidalId,
        item.externalId
      ].filter(Boolean);
      
      return keys.join('|');
    }
    
    return String(item).toLowerCase().trim();
  }

  // Add multiple items efficiently
  addAll(items) {
    for (const item of items) {
      this.add(item);
    }
  }

  // Get current false positive probability
  getCurrentFalsePositiveRate() {
    if (this.elementCount === 0) return 0;
    
    // p = (1 - e^(-k*n/m))^k
    const exponent = -(this.numHashFunctions * this.elementCount) / this.bitArraySize;
    return Math.pow(1 - Math.exp(exponent), this.numHashFunctions);
  }

  // Check if filter needs to be rebuilt
  needsRebuild() {
    return this.getCurrentFalsePositiveRate() > this.falsePositiveRate * 2;
  }

  // Serialize filter for storage
  serialize() {
    return {
      version: '1.0',
      bitArraySize: this.bitArraySize,
      numHashFunctions: this.numHashFunctions,
      elementCount: this.elementCount,
      expectedElements: this.expectedElements,
      falsePositiveRate: this.falsePositiveRate,
      bitArray: Array.from(this.bitArray),
      timestamp: Date.now()
    };
  }

  // Deserialize filter from storage
  static deserialize(data) {
    if (!data || data.version !== '1.0') {
      throw new Error('Invalid or unsupported bloom filter data');
    }
    
    const filter = new BloomFilter(data.expectedElements, data.falsePositiveRate);
    filter.bitArraySize = data.bitArraySize;
    filter.numHashFunctions = data.numHashFunctions;
    filter.elementCount = data.elementCount;
    filter.bitArray = new Uint8Array(data.bitArray);
    
    return filter;
  }

  // Get memory usage in bytes
  getMemoryUsage() {
    return this.bitArray.length + 64; // bit array + overhead
  }

  // Get statistics
  getStats() {
    return {
      bitArraySize: this.bitArraySize,
      numHashFunctions: this.numHashFunctions,
      elementCount: this.elementCount,
      memoryUsage: this.getMemoryUsage(),
      currentFalsePositiveRate: this.getCurrentFalsePositiveRate(),
      needsRebuild: this.needsRebuild()
    };
  }

  // Clear the filter
  clear() {
    this.bitArray.fill(0);
    this.elementCount = 0;
  }
}

// Export for use in extension
if (typeof module !== 'undefined' && module.exports) {
  module.exports = BloomFilter;
} else {
  window.BloomFilter = BloomFilter;
}