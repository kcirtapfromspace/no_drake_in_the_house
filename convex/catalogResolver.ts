import { v } from "convex/values";
import {
  internalAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";

// ---------------------------------------------------------------------------
// Normalization helpers
// ---------------------------------------------------------------------------

const STRIP_SUFFIXES =
  /\s*[\(\[]\s*(?:deluxe|remaster(?:ed)?|bonus|explicit|clean|radio|acoustic|live|remix|version|edit|anniversary|expanded|special|complete|original|mono|stereo|single|ep).*?[\)\]]/gi;

function normalize(s: string): string {
  return s
    .toLowerCase()
    .replace(STRIP_SUFFIXES, "")
    .replace(/\s*[\[\(].*?[\]\)]/g, "") // remaining parens/brackets
    .replace(/[^\w\s]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function normalizeTrackKey(
  trackName: string,
  artistName: string,
): string {
  return `${normalize(trackName)}||${normalize(artistName)}`;
}

function normalizeAlbumKey(albumName: string): string {
  return normalize(albumName);
}

// ---------------------------------------------------------------------------
// Internal queries
// ---------------------------------------------------------------------------

/** Get a batch of userLibraryTracks that have no canonicalTrackId set.
 *  Scans per-provider to stay within bandwidth limits. */
export const _getUnresolvedTracks = internalQuery({
  args: {
    limit: v.number(),
    userId: v.id("users"),
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const results: Array<{
      _id: Id<"userLibraryTracks">;
      trackName: string;
      albumName: string;
      artistName: string;
      artistId: string | null;
      provider: string;
      providerTrackId: string;
    }> = [];

    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )) {
      if (track.canonicalTrackId) continue;
      if (!track.trackName) continue;

      results.push({
        _id: track._id,
        trackName: track.trackName,
        albumName: track.albumName ?? "",
        artistName: track.artistName ?? "",
        artistId: track.artistId ? (track.artistId as string) : null,
        provider: track.provider,
        providerTrackId: track.providerTrackId,
      });

      if (results.length >= args.limit) break;
    }

    return results;
  },
});

/** Debug: find all artist name variations containing a substring. */
export const _findArtistNameVariations = internalQuery({
  args: { userId: v.id("users"), provider: v.string(), search: v.string() },
  handler: async (ctx, args) => {
    const searchLower = args.search.toLowerCase();
    const variations = new Map<string, number>();

    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )) {
      const name = track.artistName ?? "";
      if (name.toLowerCase().includes(searchLower)) {
        variations.set(name, (variations.get(name) ?? 0) + 1);
      }
    }

    return [...variations.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  },
});

/** Debug: count tracks with/without canonicalTrackId for a provider. */
export const _debugCounts = internalQuery({
  args: { userId: v.id("users"), provider: v.string() },
  handler: async (ctx, args) => {
    let total = 0;
    let withCanonical = 0;
    let withoutCanonical = 0;
    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )) {
      total++;
      if (track.canonicalTrackId) withCanonical++;
      else withoutCanonical++;
      if (total >= 500) break; // sample first 500
    }
    return { total, withCanonical, withoutCanonical };
  },
});

/** Get all user+provider combinations from provider connections. */
export const _getUserProviders = internalQuery({
  args: {},
  handler: async (ctx) => {
    const connections = await ctx.db.query("providerConnections").collect();
    return connections.map((c) => ({
      userId: c.userId,
      provider: c.provider,
    }));
  },
});

// ---------------------------------------------------------------------------
// Internal mutations
// ---------------------------------------------------------------------------

/** Resolve a batch of tracks to canonical records. */
export const _resolveTrackBatch = internalMutation({
  args: {
    tracks: v.array(
      v.object({
        libraryTrackId: v.id("userLibraryTracks"),
        trackName: v.string(),
        albumName: v.string(),
        artistName: v.string(),
        artistId: v.optional(v.string()),
        provider: v.string(),
        providerTrackId: v.string(),
      }),
    ),
  },
  handler: async (ctx, args) => {
    const now = new Date().toISOString();
    let created = 0;
    let linked = 0;
    let albumsCreated = 0;

    for (const t of args.tracks) {
      const key = normalizeTrackKey(t.trackName, t.artistName);

      // 1. Check if canonical track already exists
      const existing = await ctx.db
        .query("tracks")
        .withIndex("by_normalizedKey", (q) => q.eq("normalizedKey", key))
        .first();

      if (existing) {
        // Link library track to existing canonical
        await ctx.db.patch(t.libraryTrackId, {
          canonicalTrackId: existing._id,
        });

        // Update provider ID on canonical track if missing
        const providerField =
          t.provider === "spotify"
            ? "spotifyId"
            : t.provider === "apple_music"
              ? "appleMusicId"
              : t.provider === "tidal"
                ? "tidalId"
                : null;

        if (providerField && !(existing as any)[providerField]) {
          await ctx.db.patch(existing._id, {
            [providerField]: t.providerTrackId,
            updatedAt: now,
          } as any);
        }

        linked++;
        continue;
      }

      // 2. Find or create album
      let albumId: Id<"albums"> | undefined;
      if (t.albumName) {
        const albumKey = normalizeAlbumKey(t.albumName);
        const existingAlbum = await ctx.db
          .query("albums")
          .withIndex("by_normalizedTitle", (q) =>
            q.eq("normalizedTitle", albumKey),
          )
          .first();

        if (existingAlbum) {
          albumId = existingAlbum._id;
        } else {
          albumId = await ctx.db.insert("albums", {
            legacyKey: `catalog:album:${albumKey}`,
            title: t.albumName,
            normalizedTitle: albumKey,
            metadata: {},
            createdAt: now,
            updatedAt: now,
          });
          albumsCreated++;
        }

        // Link album to artist if we have an artistId
        if (t.artistId && albumId) {
          const artistDocId = t.artistId as Id<"artists">;
          const existingLink = await ctx.db
            .query("albumArtists")
            .withIndex("by_albumId", (q) => q.eq("albumId", albumId!))
            .collect()
            .then((links) =>
              links.find((l) => l.artistId === artistDocId),
            );

          if (!existingLink) {
            await ctx.db.insert("albumArtists", {
              legacyKey: `catalog:aa:${albumId}:${artistDocId}`,
              albumId,
              artistId: artistDocId,
              createdAt: now,
              updatedAt: now,
            });
          }
        }
      }

      // 3. Create canonical track
      const artistDocId = t.artistId
        ? (t.artistId as Id<"artists">)
        : undefined;

      const providerIds: Record<string, string> = {};
      if (t.provider === "spotify") providerIds.spotifyId = t.providerTrackId;
      else if (t.provider === "apple_music")
        providerIds.appleMusicId = t.providerTrackId;
      else if (t.provider === "tidal") providerIds.tidalId = t.providerTrackId;

      const trackId = await ctx.db.insert("tracks", {
        legacyKey: `catalog:track:${key}`,
        title: t.trackName,
        normalizedKey: key,
        albumId,
        artistId: artistDocId,
        metadata: {},
        createdAt: now,
        updatedAt: now,
        ...providerIds,
      } as any);

      // 4. Link library track
      await ctx.db.patch(t.libraryTrackId, { canonicalTrackId: trackId });

      // 5. Create trackCredit for the main artist
      if (artistDocId) {
        await ctx.db.insert("trackCredits", {
          legacyKey: `catalog:credit:${trackId}:${artistDocId}:main`,
          trackId,
          artistId: artistDocId,
          creditedName: t.artistName,
          role: "main",
          metadata: {},
          createdAt: now,
          updatedAt: now,
        });
      }

      // 6. Parse featuring artists from track name
      const featMatch = t.trackName.match(
        /\(?(?:feat\.?|ft\.?|featuring|with)\s+(.+?)\)?$/i,
      );
      if (featMatch) {
        const names = featMatch[1]
          .split(/[,&]/)
          .map((s) => s.trim())
          .filter((s) => s.length > 0);

        for (const name of names) {
          await ctx.db.insert("trackCredits", {
            legacyKey: `catalog:credit:${trackId}:feat:${normalize(name)}`,
            trackId,
            creditedName: name,
            role: "featured",
            metadata: {},
            createdAt: now,
            updatedAt: now,
          });
        }
      }

      created++;
    }

    return { created, linked, albumsCreated, total: args.tracks.length };
  },
});

// ---------------------------------------------------------------------------
// Public action: orchestrates the canonicalization pipeline
// ---------------------------------------------------------------------------

/** Process unresolved library tracks in batches with continuation. */
export const resolveAll = internalAction({
  args: {},
  handler: async (ctx) => {
    const BATCH_SIZE = 100;
    let totalCreated = 0;
    let totalLinked = 0;
    let rounds = 0;
    const startTime = Date.now();
    const MAX_RUNTIME_MS = 20 * 60 * 1000; // 20 min safety margin

    // Get all user+provider combinations
    const userProviders = await ctx.runQuery(
      internal.catalogResolver._getUserProviders,
      {},
    );

    for (const { userId, provider } of userProviders) {
      console.log(`[CatalogResolver] Processing ${provider} for user ${userId}`);

      while (Date.now() - startTime < MAX_RUNTIME_MS) {
        const unresolved = await ctx.runQuery(
          internal.catalogResolver._getUnresolvedTracks,
          { limit: BATCH_SIZE, userId, provider },
        );

        if (unresolved.length === 0) break;

        const result = await ctx.runMutation(
          internal.catalogResolver._resolveTrackBatch,
          {
            tracks: unresolved.map((t) => ({
              libraryTrackId: t._id,
              trackName: t.trackName,
              albumName: t.albumName,
              artistName: t.artistName,
              artistId: t.artistId ?? undefined,
              provider: t.provider,
              providerTrackId: t.providerTrackId,
            })),
          },
        );

        totalCreated += result.created;
        totalLinked += result.linked;
        rounds++;

        console.log(
          `[CatalogResolver] Round ${rounds}: ${result.created} created, ${result.linked} linked, ${result.albumsCreated} albums (${unresolved.length} processed)`,
        );

        if (unresolved.length < BATCH_SIZE) break;
      }
    }

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
    console.log(
      `[CatalogResolver] Done in ${elapsed}s: ${totalCreated} tracks created, ${totalLinked} linked, ${rounds} rounds`,
    );

    return { totalCreated, totalLinked, rounds };
  },
});

// ---------------------------------------------------------------------------
// Multi-artist linking: find tracks where artistName contains an artist name
// as a token (handles "Drake & 21 Savage", "Lil Baby, Gunna & Drake", etc.)
// ---------------------------------------------------------------------------

/** Link tracks with multi-artist names to additional artist records.
 *  Scans per provider, finds tracks where artistName contains known artist
 *  names separated by commas, ampersands, or "feat." patterns. */
export const _linkMultiArtistBatch = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
    artistId: v.id("artists"),
    artistName: v.string(),
  },
  handler: async (ctx, args) => {
    const nameLower = args.artistName.toLowerCase();
    let linked = 0;
    let catalogLinked = 0;

    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )) {
      // Skip tracks already linked to this artist
      if (track.artistId === args.artistId) continue;

      const trackArtist = (track.artistName ?? "").toLowerCase();
      if (!trackArtist.includes(nameLower)) continue;

      // Verify it's a token match (not substring of another name like "Pete Drake")
      // Split by common separators and check for exact token
      const tokens = trackArtist
        .split(/[,&]|(?:feat\.?|ft\.?|featuring|with|and)\s/i)
        .map((s) => s.trim());
      if (!tokens.some((t) => t === nameLower)) continue;

      // Link library track to this artist (secondary link via canonical track)
      if (track.canonicalTrackId) {
        const canonicalTrack = await ctx.db.get(track.canonicalTrackId);
        if (canonicalTrack) {
          // Add album-artist link if album exists
          if (canonicalTrack.albumId) {
            const existingLink = await ctx.db
              .query("albumArtists")
              .withIndex("by_albumId", (q) => q.eq("albumId", canonicalTrack.albumId!))
              .collect()
              .then((links) => links.find((l) => l.artistId === args.artistId));

            if (!existingLink) {
              const now = new Date().toISOString();
              await ctx.db.insert("albumArtists", {
                legacyKey: `catalog:aa:${canonicalTrack.albumId}:${args.artistId}`,
                albumId: canonicalTrack.albumId,
                artistId: args.artistId,
                createdAt: now,
                updatedAt: now,
              });
              catalogLinked++;
            }
          }

          // Add track credit if not already present
          const existingCredit = await ctx.db
            .query("trackCredits")
            .withIndex("by_trackId", (q) => q.eq("trackId", track.canonicalTrackId!))
            .collect()
            .then((credits) => credits.find((c) => c.artistId === args.artistId));

          if (!existingCredit) {
            const now = new Date().toISOString();
            await ctx.db.insert("trackCredits", {
              legacyKey: `catalog:credit:${track.canonicalTrackId}:${args.artistId}:main`,
              trackId: track.canonicalTrackId,
              artistId: args.artistId,
              creditedName: args.artistName,
              role: "main",
              metadata: {},
              createdAt: now,
              updatedAt: now,
            });
          }
        }
      }

      linked++;
    }

    return { linked, catalogLinked };
  },
});

/** Link multi-artist tracks for all known artists across all providers. */
export const linkMultiArtistTracks = internalAction({
  args: {
    artistId: v.optional(v.id("artists")),
  },
  handler: async (ctx, args) => {
    const userProviders = await ctx.runQuery(
      internal.catalogResolver._getUserProviders,
      {},
    );

    // If specific artist requested, just do that one
    let artists: Array<{ _id: any; canonicalName: string }>;
    if (args.artistId) {
      const artist = await ctx.runQuery(
        internal.evidenceFinder._getArtistById,
        { artistId: args.artistId },
      );
      artists = artist ? [{ _id: args.artistId, canonicalName: artist.canonicalName }] : [];
    } else {
      // Get top artists (by library presence) — limit to avoid timeout
      artists = await ctx.runQuery(
        internal.catalogResolver._getTopArtists,
        { limit: 200 },
      );
    }

    let totalLinked = 0;
    for (const artist of artists) {
      for (const { userId, provider } of userProviders) {
        const result = await ctx.runMutation(
          internal.catalogResolver._linkMultiArtistBatch,
          {
            userId,
            provider,
            artistId: artist._id,
            artistName: artist.canonicalName,
          },
        );
        totalLinked += result.linked;
        if (result.linked > 0) {
          console.log(
            `[MultiArtistLink] ${artist.canonicalName} on ${provider}: ${result.linked} tracks, ${result.catalogLinked} albums linked`,
          );
        }
      }
    }

    return { totalLinked, artistsChecked: artists.length };
  },
});

/** Get top artists by track count for multi-artist linking. */
export const _getTopArtists = internalQuery({
  args: { limit: v.number() },
  handler: async (ctx, args) => {
    const artists = await ctx.db.query("artists").take(args.limit);
    return artists.map((a) => ({
      _id: a._id,
      canonicalName: a.canonicalName,
    }));
  },
});
