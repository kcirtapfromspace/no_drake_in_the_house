export interface BloomArtist {
  id: string;
  name: string;
  spotifyId?: string;
  appleMusicId?: string;
  youtubeMusicId?: string;
  tidalId?: string;
  externalId?: string;
}

interface SerializedBloomFilter {
  version: "1.0";
  bitArraySize: number;
  numHashFunctions: number;
  elementCount: number;
  expectedElements: number;
  falsePositiveRate: number;
  bitArray: number[];
  timestamp: number;
}

class BloomFilter {
  expectedElements: number;
  falsePositiveRate: number;
  bitArraySize: number;
  numHashFunctions: number;
  bitArray: Uint8Array;
  elementCount: number;

  constructor(expectedElements = 10000, falsePositiveRate = 0.01) {
    this.expectedElements = expectedElements;
    this.falsePositiveRate = falsePositiveRate;
    this.bitArraySize = Math.ceil(
      -(expectedElements * Math.log(falsePositiveRate)) /
        (Math.log(2) * Math.log(2)),
    );
    this.numHashFunctions = Math.ceil(
      (this.bitArraySize / expectedElements) * Math.log(2),
    );
    this.bitArray = new Uint8Array(Math.ceil(this.bitArraySize / 8));
    this.elementCount = 0;
  }

  private hash1(value: string) {
    let hash = 2166136261;
    for (let index = 0; index < value.length; index += 1) {
      hash ^= value.charCodeAt(index);
      hash = Math.imul(hash, 16777619);
    }
    return Math.abs(hash) % this.bitArraySize;
  }

  private hash2(value: string) {
    let hash = 5381;
    for (let index = 0; index < value.length; index += 1) {
      hash = (Math.imul(hash, 33) + value.charCodeAt(index)) | 0;
    }
    return Math.abs(hash) % this.bitArraySize;
  }

  private getHashValues(value: string) {
    const h1 = this.hash1(value);
    const h2 = this.hash2(value);

    return Array.from({ length: this.numHashFunctions }, (_, index) => {
      return (h1 + index * h2) % this.bitArraySize;
    });
  }

  private setBit(index: number) {
    const byteIndex = Math.floor(index / 8);
    const bitIndex = index % 8;
    this.bitArray[byteIndex] |= 1 << bitIndex;
  }

  add(value: string) {
    for (const hash of this.getHashValues(value)) {
      this.setBit(hash);
    }
    this.elementCount += 1;
  }

  serialize(): SerializedBloomFilter {
    return {
      version: "1.0",
      bitArraySize: this.bitArraySize,
      numHashFunctions: this.numHashFunctions,
      elementCount: this.elementCount,
      expectedElements: this.expectedElements,
      falsePositiveRate: this.falsePositiveRate,
      bitArray: Array.from(this.bitArray),
      timestamp: Date.now(),
    };
  }
}

function normalizeArtistIdentifiers(artist: BloomArtist) {
  const identifiers = [
    artist.name?.toLowerCase().trim(),
    artist.spotifyId,
    artist.appleMusicId,
    artist.youtubeMusicId,
    artist.tidalId,
    artist.externalId,
  ].filter((value): value is string => Boolean(value && value.trim()));

  return identifiers;
}

export function buildSerializedBloomFilter(artists: BloomArtist[]) {
  const filter = new BloomFilter(Math.max(artists.length * 2, 1000), 0.01);

  for (const artist of artists) {
    for (const identifier of normalizeArtistIdentifiers(artist)) {
      filter.add(identifier);
    }
  }

  return filter.serialize();
}
