import { v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { buildSerializedBloomFilter } from "./lib/bloom";
import { nowIso } from "./lib/auth";

async function buildArtistPayload(ctx: any) {
  const [artists, offenses, blocks] = await Promise.all([
    ctx.db.query("artists").collect(),
    ctx.db.query("artistOffenses").collect(),
    ctx.db.query("userArtistBlocks").collect(),
  ]);

  const includedArtistIds = new Set<string>();
  offenses.forEach((offense: any) => includedArtistIds.add(offense.artistId));
  blocks.forEach((block: any) => includedArtistIds.add(block.artistId));

  return artists
    .filter((artist: any) => includedArtistIds.has(artist._id))
    .map((artist: any) => ({
      id: artist._id,
      name: artist.canonicalName,
      spotifyId:
        typeof artist.externalIds?.spotify === "string"
          ? artist.externalIds.spotify
          : undefined,
      appleMusicId:
        typeof artist.externalIds?.apple === "string"
          ? artist.externalIds.apple
          : undefined,
      youtubeMusicId:
        typeof artist.externalIds?.youtube_music === "string"
          ? artist.externalIds.youtube_music
          : undefined,
      tidalId:
        typeof artist.externalIds?.tidal === "string"
          ? artist.externalIds.tidal
          : undefined,
      externalId: artist.legacyArtistId ?? artist.legacyKey,
    }));
}

export const refreshGlobalSnapshot = mutation({
  args: {},
  handler: async (ctx) => {
    const artists = await buildArtistPayload(ctx);
    const payload = {
      version: Date.now(),
      timestamp: Date.now(),
      artists,
      bloomFilter: buildSerializedBloomFilter(artists),
    };

    const existing = await ctx.db
      .query("bloomFilterSnapshots")
      .withIndex("by_scope", (q) => q.eq("scope", "global"))
      .unique();

    if (existing) {
      await ctx.db.patch(existing._id, {
        version: payload.version,
        artistCount: artists.length,
        payload,
        signature: undefined,
        generatedAt: nowIso(),
        updatedAt: nowIso(),
      });
      return await ctx.db.get(existing._id);
    }

    const insertedId = await ctx.db.insert("bloomFilterSnapshots", {
      legacyKey: "runtime:snapshot:global",
      scope: "global",
      version: payload.version,
      artistCount: artists.length,
      payload,
      signature: undefined,
      generatedAt: nowIso(),
      createdAt: nowIso(),
      updatedAt: nowIso(),
    });

    return await ctx.db.get(insertedId);
  },
});

export const getLatestPublicSnapshot = query({
  args: {},
  handler: async (ctx) => {
    const existing = await ctx.db
      .query("bloomFilterSnapshots")
      .withIndex("by_scope", (q) => q.eq("scope", "global"))
      .unique();

    if (existing) {
      return existing.payload;
    }

    const artists = await buildArtistPayload(ctx);
    return {
      version: Date.now(),
      timestamp: Date.now(),
      artists,
      bloomFilter: buildSerializedBloomFilter(artists),
    };
  },
});
