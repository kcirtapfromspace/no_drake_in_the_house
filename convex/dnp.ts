import { ConvexError, v } from "convex/values";
import type { Doc, Id } from "./_generated/dataModel";
import { action, mutation, query, type MutationCtx } from "./_generated/server";
import { api } from "./_generated/api";
import { nowIso, requireCurrentUser } from "./lib/auth";
import {
  decryptToken,
  getEncryptionKey,
  isEncrypted,
} from "./lib/crypto";

async function resolveArtistByQuery(ctx: any, searchQuery: string) {
  const matches = await ctx.db
    .query("artists")
    .withSearchIndex("search_canonicalName", (q: any) =>
      q.search("canonicalName", searchQuery),
    )
    .take(10);

  const exact = matches.find(
    (artist: Doc<"artists">) =>
      artist.canonicalName.toLowerCase() === searchQuery.toLowerCase(),
  );

  return exact ?? matches[0] ?? null;
}

async function hydrateEntries(ctx: any, blocks: Doc<"userArtistBlocks">[]) {
  const entries = await Promise.all(
    blocks.map(async (block) => {
      const artist = await ctx.db.get(block.artistId);
      if (!artist) {
        return null;
      }

      return {
        artist: {
          id: artist._id,
          canonical_name: artist.canonicalName,
          external_ids: artist.externalIds ?? {},
          metadata: artist.metadata ?? {},
        },
        tags: block.tags ?? [],
        note: block.note,
        created_at: block.createdAt,
      };
    }),
  );

  return entries.filter(Boolean);
}

export const listCurrentUser = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const blocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    return await hydrateEntries(ctx, blocks);
  },
});

export const searchArtists = query({
  args: {
    query: v.string(),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const matches = await ctx.db
      .query("artists")
      .withSearchIndex("search_canonicalName", (q) =>
        q.search("canonicalName", args.query),
      )
      .take(args.limit ?? 20);

    const artists = await Promise.all(
      matches.map(async (artist) => {
        const offenses = await ctx.db
          .query("artistOffenses")
          .withIndex("by_artistId", (q) => q.eq("artistId", artist._id))
          .collect();

        return {
          id: artist._id,
          canonical_name: artist.canonicalName,
          genres:
            Array.isArray((artist.metadata as Record<string, unknown> | undefined)?.genres)
              ? ((artist.metadata as Record<string, unknown>).genres as string[])
              : [],
          image_url:
            typeof (artist.metadata as Record<string, unknown> | undefined)?.image ===
            "string"
              ? ((artist.metadata as Record<string, unknown>).image as string)
              : undefined,
          offense_count: offenses.length,
          has_offenses: offenses.length > 0,
          source: "convex",
        };
      }),
    );

    return {
      artists,
      total: artists.length,
      sources: {
        local: artists.length,
        catalog: 0,
      },
    };
  },
});

export const addArtistBlock = mutation({
  args: {
    artistId: v.optional(v.id("artists")),
    query: v.optional(v.string()),
    tags: v.optional(v.array(v.string())),
    note: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const artist =
      args.artistId !== undefined
        ? await ctx.db.get(args.artistId)
        : args.query
          ? await resolveArtistByQuery(ctx, args.query)
          : null;

    if (!artist) {
      throw new ConvexError("Artist not found.");
    }

    const existing = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((blocks: Doc<"userArtistBlocks">[]) =>
        blocks.find((block) => block.artistId === artist._id) ?? null,
      );

    const payload = {
      tags: args.tags ?? [],
      note: args.note,
      source: "runtime",
      updatedAt: nowIso(),
    };

    if (existing) {
      await ctx.db.patch(existing._id, payload);
    } else {
      await ctx.db.insert("userArtistBlocks", {
        legacyKey: `runtime:block:${user._id}:${artist._id}`,
        userId: user._id,
        artistId: artist._id,
        tags: args.tags ?? [],
        note: args.note,
        source: "runtime",
        createdAt: nowIso(),
        updatedAt: nowIso(),
      });
    }

    const blocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((entries: Doc<"userArtistBlocks">[]) =>
        entries.filter((entry) => entry.artistId === artist._id),
      );

    return (await hydrateEntries(ctx, blocks))[0] ?? null;
  },
});

export const updateArtistBlock = mutation({
  args: {
    artistId: v.id("artists"),
    tags: v.optional(v.array(v.string())),
    note: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((entries: Doc<"userArtistBlocks">[]) =>
        entries.find((entry) => entry.artistId === args.artistId) ?? null,
      );

    if (!existing) {
      throw new ConvexError("DNP entry not found.");
    }

    await ctx.db.patch(existing._id, {
      tags: args.tags ?? [],
      note: args.note,
      updatedAt: nowIso(),
    });

    return (await hydrateEntries(ctx, [existing]))[0] ?? null;
  },
});

export const importBlocklist = mutation({
  args: {
    entries: v.array(
      v.object({
        artistName: v.string(),
        tags: v.optional(v.array(v.string())),
        note: v.optional(v.string()),
      }),
    ),
  },
  handler: async (ctx: MutationCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    let imported = 0;
    let skipped = 0;

    for (const entry of args.entries) {
      const artist = await resolveArtistByQuery(ctx, entry.artistName);
      if (!artist) {
        skipped++;
        continue;
      }

      const existing = await ctx.db
        .query("userArtistBlocks")
        .withIndex("by_user_artist", (q) =>
          q.eq("userId", user._id).eq("artistId", artist._id),
        )
        .unique();

      if (!existing) {
        await ctx.db.insert("userArtistBlocks", {
          legacyKey: `runtime:block:${user._id}:${artist._id}`,
          userId: user._id,
          artistId: artist._id,
          tags: entry.tags ?? [],
          note: entry.note,
          source: "import",
          createdAt: nowIso(),
          updatedAt: nowIso(),
        });
        imported++;
      } else {
        skipped++;
      }
    }

    return { imported, skipped, total: args.entries.length };
  },
});

export const exportBlocklist = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const blocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const entries = await Promise.all(
      blocks.map(async (block) => {
        const artist = await ctx.db.get(block.artistId);
        return {
          artist_name: artist?.canonicalName ?? "Unknown",
          artist_id: block.artistId,
          tags: block.tags ?? [],
          note: block.note,
          source: block.source,
          created_at: block.createdAt,
        };
      }),
    );

    return {
      entries,
      total: entries.length,
      exported_at: nowIso(),
    };
  },
});

export const removeArtistBlock = mutation({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((entries: Doc<"userArtistBlocks">[]) =>
        entries.find((entry) => entry.artistId === args.artistId) ?? null,
      );

    if (!existing) {
      return { success: true };
    }

    await ctx.db.delete(existing._id);
    return { success: true };
  },
});

// ---------------------------------------------------------------------------
// Spotify Artist Search
// ---------------------------------------------------------------------------

/**
 * Search Spotify's catalog for artists that may not yet exist in the local database.
 * Uses the user's Spotify OAuth token to call /v1/search?type=artist.
 * Returns matches with external IDs so they can be imported and blocked.
 */
export const searchSpotifyArtists = action({
  args: {
    query: v.string(),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const connection: any = await ctx.runQuery(
      api.enforcement._getConnection,
      { provider: "spotify" },
    );

    if (!connection || !connection.accessToken) {
      throw new ConvexError(
        "No active Spotify connection. Please connect your account first.",
      );
    }

    const encryptionKey = getEncryptionKey();
    const accessToken = isEncrypted(connection.accessToken)
      ? await decryptToken(connection.accessToken, encryptionKey)
      : connection.accessToken;

    const limit = Math.min(args.limit ?? 10, 20);
    const searchUrl = `https://api.spotify.com/v1/search?type=artist&q=${encodeURIComponent(args.query)}&limit=${limit}`;

    const resp = await fetch(searchUrl, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });

    if (!resp.ok) {
      const errText = await resp.text();
      throw new ConvexError(
        `Spotify search failed (${resp.status}): ${errText.substring(0, 200)}`,
      );
    }

    const data = (await resp.json()) as {
      artists?: {
        items?: Array<{
          id: string;
          name: string;
          genres?: string[];
          images?: Array<{ url: string }>;
          popularity?: number;
          followers?: { total: number };
        }>;
      };
    };

    const items = data.artists?.items ?? [];

    return {
      artists: items.map((artist) => ({
        spotifyId: artist.id,
        name: artist.name,
        genres: artist.genres ?? [],
        imageUrl: artist.images?.[0]?.url,
        popularity: artist.popularity ?? 0,
        followers: artist.followers?.total ?? 0,
      })),
      total: items.length,
    };
  },
});

/**
 * Import an artist from a Spotify search result into the local database
 * and optionally block them in one step.
 */
export const importAndBlockSpotifyArtist = action({
  args: {
    spotifyId: v.string(),
    name: v.string(),
    genres: v.optional(v.array(v.string())),
    imageUrl: v.optional(v.string()),
    tags: v.optional(v.array(v.string())),
    note: v.optional(v.string()),
    blockAfterImport: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    // Check if artist already exists by Spotify external ID
    const artistId: any = await ctx.runMutation(
      api.dnp._upsertSpotifyArtist,
      {
        spotifyId: args.spotifyId,
        name: args.name,
        genres: args.genres ?? [],
        imageUrl: args.imageUrl,
      },
    );

    if (!artistId) {
      throw new ConvexError("Failed to import artist.");
    }

    if (args.blockAfterImport !== false) {
      const _blockResult: any = await ctx.runMutation(api.dnp.addArtistBlock, {
        artistId,
        tags: args.tags ?? [],
        note: args.note,
      });
    }

    return { artistId, blocked: args.blockAfterImport !== false };
  },
});

/**
 * Internal: upsert an artist from Spotify data.
 * Returns the Convex artist ID.
 */
export const _upsertSpotifyArtist = mutation({
  args: {
    spotifyId: v.string(),
    name: v.string(),
    genres: v.array(v.string()),
    imageUrl: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    // Check for existing artist by canonical name or Spotify ID
    const existingByName = await ctx.db
      .query("artists")
      .withSearchIndex("search_canonicalName", (q) =>
        q.search("canonicalName", args.name),
      )
      .take(5);

    const exactMatch = existingByName.find(
      (a) =>
        a.canonicalName.toLowerCase() === args.name.toLowerCase() ||
        (a.externalIds as Record<string, string> | undefined)?.spotify === args.spotifyId,
    );

    if (exactMatch) {
      // Update external IDs if missing
      const existingExternalIds = (exactMatch.externalIds ?? {}) as Record<string, string>;
      if (!existingExternalIds.spotify) {
        await ctx.db.patch(exactMatch._id, {
          externalIds: { ...existingExternalIds, spotify: args.spotifyId },
          metadata: {
            ...(exactMatch.metadata as Record<string, unknown> ?? {}),
            genres: args.genres,
            image: args.imageUrl,
          },
          updatedAt: nowIso(),
        });
      }
      return exactMatch._id;
    }

    // Create new artist
    const now = nowIso();
    return await ctx.db.insert("artists", {
      legacyKey: `spotify:${args.spotifyId}`,
      canonicalName: args.name,
      externalIds: { spotify: args.spotifyId },
      metadata: {
        genres: args.genres,
        image: args.imageUrl,
        source: "spotify_search",
      },
      aliases: [],
      status: "active",
      createdAt: now,
      updatedAt: now,
    });
  },
});
