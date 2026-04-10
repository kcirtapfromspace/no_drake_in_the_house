import { v } from "convex/values";
import {
  internalAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";
// Client credentials flow used for catalog enrichment (no user tokens needed)

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAX_RETRIES = 3;
const MAX_RETRY_WAIT_SECS = 60;
const SPOTIFY_PAGE_SIZE = 50;
const SAFE_RUNTIME_MS = 20 * 60 * 1000; // 20 min

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function spotifyFetch(url: string, token: string): Promise<Response> {
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 30_000);
    let res: Response;
    try {
      res = await fetch(url, {
        headers: { Authorization: `Bearer ${token}`, Accept: "application/json" },
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timeout);
    }
    if (res.status === 429) {
      if (attempt === MAX_RETRIES) throw new Error(`Spotify 429 after ${MAX_RETRIES} retries`);
      const wait = Math.min(parseInt(res.headers.get("retry-after") || "2", 10) || 2, MAX_RETRY_WAIT_SECS);
      await new Promise((r) => setTimeout(r, wait * 1000));
      continue;
    }
    return res;
  }
  throw new Error("unreachable");
}

// ---------------------------------------------------------------------------
// Internal queries
// ---------------------------------------------------------------------------

/** Get a Spotify access token via client credentials (no user auth needed). */
async function getSpotifyClientToken(): Promise<string | null> {
  const clientId = process.env.SPOTIFY_CLIENT_ID;
  const clientSecret = process.env.SPOTIFY_CLIENT_SECRET;
  if (!clientId || !clientSecret) return null;

  const res = await fetch("https://accounts.spotify.com/api/token", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: `grant_type=client_credentials&client_id=${clientId}&client_secret=${clientSecret}`,
  });

  if (!res.ok) return null;
  const data = await res.json();
  return data.access_token ?? null;
}

/** Get artists that need catalog enrichment. */
export const _getUnenrichedArtists = internalQuery({
  args: { limit: v.number() },
  handler: async (ctx, args) => {
    const results: Array<{
      _id: Id<"artists">;
      canonicalName: string;
      spotifyId: string | null;
    }> = [];

    for await (const artist of ctx.db.query("artists")) {
      if (artist.catalogEnrichedAt) continue;

      const externalIds = (artist.externalIds ?? {}) as Record<string, string>;
      results.push({
        _id: artist._id,
        canonicalName: artist.canonicalName,
        spotifyId: externalIds.spotify ?? null,
      });

      if (results.length >= args.limit) break;
    }

    return results;
  },
});

// ---------------------------------------------------------------------------
// Internal mutations
// ---------------------------------------------------------------------------

/** Store Spotify artist ID on an artist record. */
export const _setSpotifyId = internalMutation({
  args: {
    artistId: v.id("artists"),
    spotifyId: v.string(),
  },
  handler: async (ctx, args) => {
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return;
    const externalIds = (artist.externalIds ?? {}) as Record<string, string>;
    externalIds.spotify = args.spotifyId;
    await ctx.db.patch(args.artistId, {
      externalIds,
      updatedAt: new Date().toISOString(),
    });
  },
});

/** Ingest albums and tracks from Spotify into the golden catalog. */
export const _ingestSpotifyAlbums = internalMutation({
  args: {
    artistId: v.id("artists"),
    artistName: v.string(),
    albums: v.array(
      v.object({
        spotifyId: v.string(),
        name: v.string(),
        releaseDate: v.optional(v.string()),
        albumType: v.optional(v.string()),
        imageUrl: v.optional(v.string()),
        tracks: v.array(
          v.object({
            spotifyId: v.string(),
            name: v.string(),
            trackNumber: v.number(),
            duration: v.number(),
            isrc: v.optional(v.string()),
            artists: v.array(
              v.object({
                spotifyId: v.string(),
                name: v.string(),
              }),
            ),
          }),
        ),
      }),
    ),
  },
  handler: async (ctx, args) => {
    const now = new Date().toISOString();
    let albumsCreated = 0;
    let tracksCreated = 0;
    let creditsCreated = 0;

    for (const album of args.albums) {
      // Upsert album by spotifyId
      let albumDoc = await ctx.db
        .query("albums")
        .withIndex("by_spotifyId", (q) => q.eq("spotifyId", album.spotifyId))
        .first();

      let albumId: Id<"albums">;
      if (albumDoc) {
        albumId = albumDoc._id;
        // Update metadata if we have new info
        if (album.imageUrl) {
          const meta = (albumDoc.metadata ?? {}) as Record<string, any>;
          if (!meta.coverUrl) {
            await ctx.db.patch(albumId, {
              metadata: { ...meta, coverUrl: album.imageUrl },
              updatedAt: now,
            });
          }
        }
      } else {
        const normalizedTitle = album.name
          .toLowerCase()
          .replace(/\s*[\(\[].*?[\)\]]/g, "")
          .replace(/[^\w\s]/g, "")
          .replace(/\s+/g, " ")
          .trim();

        albumId = await ctx.db.insert("albums", {
          legacyKey: `spotify:album:${album.spotifyId}`,
          title: album.name,
          normalizedTitle,
          spotifyId: album.spotifyId,
          releaseDate: album.releaseDate,
          metadata: {
            albumType: album.albumType,
            coverUrl: album.imageUrl,
          },
          createdAt: now,
          updatedAt: now,
        });
        albumsCreated++;
      }

      // Link album to artist
      const existingLink = await ctx.db
        .query("albumArtists")
        .withIndex("by_albumId", (q) => q.eq("albumId", albumId))
        .collect()
        .then((links) => links.find((l) => l.artistId === args.artistId));

      if (!existingLink) {
        await ctx.db.insert("albumArtists", {
          legacyKey: `spotify:aa:${albumId}:${args.artistId}`,
          albumId,
          artistId: args.artistId,
          createdAt: now,
          updatedAt: now,
        });
      }

      // Upsert tracks
      for (const track of album.tracks) {
        let trackDoc = await ctx.db
          .query("tracks")
          .withIndex("by_spotifyId", (q) => q.eq("spotifyId", track.spotifyId))
          .first();

        let trackId: Id<"tracks">;
        if (trackDoc) {
          trackId = trackDoc._id;
          // Update missing fields
          const patches: Record<string, any> = {};
          if (!trackDoc.isrc && track.isrc) patches.isrc = track.isrc;
          if (!trackDoc.duration && track.duration) patches.duration = track.duration;
          if (!trackDoc.trackNumber) patches.trackNumber = track.trackNumber;
          if (!trackDoc.albumId) patches.albumId = albumId;
          if (!trackDoc.artistId) patches.artistId = args.artistId;
          if (Object.keys(patches).length > 0) {
            await ctx.db.patch(trackId, { ...patches, updatedAt: now });
          }
        } else {
          const normalizedKey = `${track.name.toLowerCase().replace(/[^\w\s]/g, "").replace(/\s+/g, " ").trim()}||${args.artistName.toLowerCase().trim()}`;

          // Check if a track with this normalized key already exists (from library import)
          const existingByKey = await ctx.db
            .query("tracks")
            .withIndex("by_normalizedKey", (q) => q.eq("normalizedKey", normalizedKey))
            .first();

          if (existingByKey) {
            trackId = existingByKey._id;
            const patches: Record<string, any> = { spotifyId: track.spotifyId, updatedAt: now };
            if (!existingByKey.isrc && track.isrc) patches.isrc = track.isrc;
            if (!existingByKey.duration && track.duration) patches.duration = track.duration;
            if (!existingByKey.trackNumber) patches.trackNumber = track.trackNumber;
            if (!existingByKey.albumId) patches.albumId = albumId;
            if (!existingByKey.artistId) patches.artistId = args.artistId;
            await ctx.db.patch(trackId, patches);
          } else {
            trackId = await ctx.db.insert("tracks", {
              legacyKey: `spotify:track:${track.spotifyId}`,
              title: track.name,
              normalizedKey,
              albumId,
              artistId: args.artistId,
              spotifyId: track.spotifyId,
              isrc: track.isrc,
              duration: track.duration,
              trackNumber: track.trackNumber,
              metadata: {},
              createdAt: now,
              updatedAt: now,
            } as any);
            tracksCreated++;
          }
        }

        // Create track credits for each artist on this track
        for (const artist of track.artists) {
          const isMainArtist = artist.name.toLowerCase() === args.artistName.toLowerCase();
          const existingCredit = await ctx.db
            .query("trackCredits")
            .withIndex("by_trackId", (q) => q.eq("trackId", trackId))
            .collect()
            .then((credits) =>
              credits.find((c) => c.creditedName.toLowerCase() === artist.name.toLowerCase()),
            );

          if (!existingCredit) {
            await ctx.db.insert("trackCredits", {
              legacyKey: `spotify:credit:${trackId}:${artist.spotifyId}`,
              trackId,
              creditedName: artist.name,
              role: isMainArtist ? "main" : "featured",
              metadata: { spotifyArtistId: artist.spotifyId },
              createdAt: now,
              updatedAt: now,
            });
            creditsCreated++;
          }
        }
      }
    }

    return { albumsCreated, tracksCreated, creditsCreated };
  },
});

/** Mark artist as catalog-enriched. */
export const _markEnriched = internalMutation({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => {
    await ctx.db.patch(args.artistId, {
      catalogEnrichedAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    });
  },
});

// ---------------------------------------------------------------------------
// Main enrichment action
// ---------------------------------------------------------------------------

/** Enrich a single artist's catalog from Spotify. */
export const enrichArtist = internalAction({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => {
    // 1. Get artist info
    const artist = await ctx.runQuery(internal.evidenceFinder._getArtistById, {
      artistId: args.artistId,
    });
    if (!artist) throw new Error("Artist not found");

    // 2. Get Spotify token via client credentials (no user auth needed)
    const accessToken = await getSpotifyClientToken();
    if (!accessToken) {
      console.log("[CatalogEnrichment] Could not get Spotify client credentials token — check SPOTIFY_CLIENT_ID/SECRET env vars");
      return { status: "no_token" };
    }

    // 3. Resolve Spotify artist ID if missing
    const externalIds = (artist.externalIds ?? {}) as Record<string, string>;
    let spotifyArtistId = externalIds.spotify;

    if (!spotifyArtistId) {
      const searchUrl = `https://api.spotify.com/v1/search?q=${encodeURIComponent(artist.canonicalName)}&type=artist&limit=1`;
      const searchRes = await spotifyFetch(searchUrl, accessToken);
      if (searchRes.ok) {
        const data = await searchRes.json();
        const match = data.artists?.items?.[0];
        if (match) {
          spotifyArtistId = match.id;
          await ctx.runMutation(internal.catalogEnrichment._setSpotifyId, {
            artistId: args.artistId,
            spotifyId: spotifyArtistId,
          });
          console.log(`[CatalogEnrichment] Resolved ${artist.canonicalName} → Spotify:${spotifyArtistId}`);
        }
      }
    }

    if (!spotifyArtistId) {
      console.log(`[CatalogEnrichment] Could not find Spotify ID for ${artist.canonicalName}`);
      await ctx.runMutation(internal.catalogEnrichment._markEnriched, { artistId: args.artistId });
      return { status: "no_spotify_id" };
    }

    // 4. Fetch all albums
    const allAlbums: any[] = [];
    let offset = 0;

    while (true) {
      const url = `https://api.spotify.com/v1/artists/${spotifyArtistId}/albums?limit=20&offset=${offset}`;
      console.log(`[CatalogEnrichment] Fetching: ${url}`);
      const res = await spotifyFetch(url, accessToken);
      console.log(`[CatalogEnrichment] Response status: ${res.status}`);
      if (!res.ok) {
        const errBody = await res.text().catch(() => "");
        console.error(`[CatalogEnrichment] Spotify albums ${res.status} for ${artist.canonicalName}: ${errBody.slice(0, 300)}`);
        break;
      }
      const data = await res.json();
      const albums = data.items ?? [];
      allAlbums.push(...albums);

      if (!data.next || albums.length < 20) break;
      offset += 20;
    }

    console.log(`[CatalogEnrichment] ${artist.canonicalName}: ${allAlbums.length} albums found`);

    // 5. Fetch tracks for each album (in batches to stay within mutation limits)
    const ALBUM_BATCH = 5;
    let totalTracksCreated = 0;
    let totalAlbumsCreated = 0;

    for (let i = 0; i < allAlbums.length; i += ALBUM_BATCH) {
      const batch = allAlbums.slice(i, i + ALBUM_BATCH);
      const enrichedAlbums: any[] = [];

      for (const album of batch) {
        // Fetch tracks for this album
        const tracksUrl = `https://api.spotify.com/v1/albums/${album.id}/tracks?limit=${SPOTIFY_PAGE_SIZE}`;
        const tracksRes = await spotifyFetch(tracksUrl, accessToken);
        if (!tracksRes.ok) continue;
        const tracksData = await tracksRes.json();

        const image = album.images?.[0]?.url ?? null;

        enrichedAlbums.push({
          spotifyId: album.id,
          name: album.name,
          releaseDate: album.release_date,
          albumType: album.album_type,
          imageUrl: image,
          tracks: (tracksData.items ?? []).map((t: any) => ({
            spotifyId: t.id,
            name: t.name,
            trackNumber: t.track_number ?? 0,
            duration: t.duration_ms ?? 0,
            isrc: t.external_ids?.isrc,
            artists: (t.artists ?? []).map((a: any) => ({
              spotifyId: a.id,
              name: a.name,
            })),
          })),
        });

        // Small delay between album track fetches
        await new Promise((r) => setTimeout(r, 200));
      }

      if (enrichedAlbums.length > 0) {
        const result = await ctx.runMutation(
          internal.catalogEnrichment._ingestSpotifyAlbums,
          {
            artistId: args.artistId,
            artistName: artist.canonicalName,
            albums: enrichedAlbums,
          },
        );
        totalAlbumsCreated += result.albumsCreated;
        totalTracksCreated += result.tracksCreated;
      }
    }

    // 6. Mark as enriched
    await ctx.runMutation(internal.catalogEnrichment._markEnriched, { artistId: args.artistId });

    console.log(
      `[CatalogEnrichment] ${artist.canonicalName}: ${totalAlbumsCreated} albums, ${totalTracksCreated} tracks created`,
    );

    return {
      status: "enriched",
      albums: allAlbums.length,
      albumsCreated: totalAlbumsCreated,
      tracksCreated: totalTracksCreated,
    };
  },
});

/** Batch enrich unenriched artists. */
export const enrichBatch = internalAction({
  args: {},
  handler: async (ctx): Promise<{ enriched: number; total: number }> => {
    const BATCH_SIZE = 10;
    const artists: Array<{ _id: Id<"artists">; canonicalName: string; spotifyId: string | null }> =
      await ctx.runQuery(
        internal.catalogEnrichment._getUnenrichedArtists,
        { limit: BATCH_SIZE },
      );

    if (artists.length === 0) {
      console.log("[CatalogEnrichment] No unenriched artists found");
      return { enriched: 0, total: 0 };
    }

    let enriched = 0;
    for (const artist of artists) {
      try {
        await ctx.scheduler.runAfter(enriched * 5000, internal.catalogEnrichment.enrichArtist, {
          artistId: artist._id,
        });
        enriched++;
      } catch (e: any) {
        console.error(`[CatalogEnrichment] Failed to schedule ${artist.canonicalName}: ${e.message}`);
      }
    }

    return { enriched, total: artists.length };
  },
});
