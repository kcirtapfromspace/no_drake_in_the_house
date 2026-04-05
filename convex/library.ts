import { ConvexError, v } from "convex/values";
import type { Doc } from "./_generated/dataModel";
import { internalMutation, mutation, query } from "./_generated/server";
import type { MutationCtx, QueryCtx } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const scanLibrary = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const scans = await ctx.db
      .query("libraryScans")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const latest = scans.sort((a, b) =>
      b.scanStartedAt.localeCompare(a.scanStartedAt),
    )[0];

    return latest ?? null;
  },
});

export const listTracks = query({
  args: {
    provider: v.optional(v.string()),
    limit: v.optional(v.number()),
    offset: v.optional(v.number()),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    let tracksQuery = args.provider
      ? ctx.db
          .query("userLibraryTracks")
          .withIndex("by_user_provider", (q) =>
            q.eq("userId", user._id).eq("provider", args.provider!),
          )
      : ctx.db
          .query("userLibraryTracks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id));

    const offset = args.offset ?? 0;
    const limit = Math.min(args.limit ?? 50, 2000);
    let skipped = 0;
    const paginated: Array<{
      id: string;
      provider: string;
      provider_track_id: string;
      track_name?: string;
      album_name?: string;
      artist_id?: any;
      artist_name?: string;
      source_type?: string;
      playlist_name?: string;
      added_at?: string;
    }> = [];

    for await (const t of tracksQuery) {
      if (skipped < offset) {
        skipped++;
        continue;
      }
      paginated.push({
        id: t._id,
        provider: t.provider,
        provider_track_id: t.providerTrackId,
        track_name: t.trackName,
        album_name: t.albumName,
        artist_id: t.artistId,
        artist_name: t.artistName,
        source_type: t.sourceType,
        playlist_name: t.playlistName,
        added_at: t.addedAt,
      });
      if (paginated.length >= limit) break;
    }

    return {
      tracks: paginated,
      total: offset + paginated.length + (paginated.length >= limit ? 1 : 0),
      offset,
      limit,
    };
  },
});

/**
 * Return aggregate counts (songs, albums, artists, playlists) for a provider
 * without transferring every track document to the client.
 */
export const getLibraryStats = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    // Use async iteration instead of .collect() to avoid materializing all
    // track documents at once. Each document is still read from the DB, but
    // we avoid holding the entire result set in memory.
    let songs = 0;
    let explicitAlbums = 0;
    let explicitArtists = 0;
    let playlistEntries = 0;
    let totalItems = 0;
    let lastSynced = "";
    const playlistNames = new Set<string>();
    const uniqueAlbums = new Set<string>();
    const uniqueArtists = new Set<string>();

    for await (const t of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )) {
      totalItems++;
      const st = (t.sourceType ?? "").toLowerCase();
      if (t.playlistName) playlistNames.add(t.playlistName);

      if (t.albumName) uniqueAlbums.add(t.albumName);
      if (t.artistName && t.artistName !== "Unknown Artist") uniqueArtists.add(t.artistName);

      if (
        st === "playlist_track" ||
        st === "playlist_item"
      ) {
        // Playlist tracks are counted separately under "playlists", not
        // as songs, so the songs column reflects liked/library tracks only.
      } else if (
        st === "favorite_track" ||
        st === "liked" ||
        st === "liked_video" ||
        st === "library_song" ||
        st === "library" ||
        st === ""
      ) {
        songs++;
      } else if (
        st === "favorite_album" ||
        st === "saved_album" ||
        st === "library_album"
      ) {
        explicitAlbums++;
      } else if (
        st === "favorite_artist" ||
        st === "followed_artist" ||
        st === "subscription"
      ) {
        explicitArtists++;
      }

      if (st.includes("playlist") || st === "library_playlist") {
        playlistEntries++;
      }

      const ts = t.lastSyncedAt ?? t.createdAt ?? "";
      if (ts > lastSynced) lastSynced = ts;
    }

    const albums = explicitAlbums > 0 ? explicitAlbums : uniqueAlbums.size;
    const artists = explicitArtists > 0 ? explicitArtists : uniqueArtists.size;
    const playlists = playlistNames.size > 0 ? playlistNames.size : playlistEntries;

    return {
      songs,
      albums,
      artists,
      playlists,
      totalItems,
      lastSynced: lastSynced || null,
    };
  },
});

export const listItems = query({
  args: {
    provider: v.optional(v.string()),
    limit: v.optional(v.number()),
    offset: v.optional(v.number()),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const limit = Math.min(args.limit ?? 200, 2000);
    const offset = args.offset ?? 0;

    const baseQuery = args.provider
      ? ctx.db
          .query("userLibraryTracks")
          .withIndex("by_user_provider", (q) =>
            q.eq("userId", user._id).eq("provider", args.provider!),
          )
      : ctx.db
          .query("userLibraryTracks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id));

    // Skip `offset` rows, then take `limit`
    let skipped = 0;
    const items: Array<{
      id: string;
      provider: string;
      track_name?: string;
      artist_name?: string;
      album_name?: string;
    }> = [];

    for await (const t of baseQuery) {
      if (skipped < offset) {
        skipped++;
        continue;
      }
      items.push({
        id: t._id,
        provider: t.provider,
        track_name: t.trackName,
        artist_name: t.artistName,
        album_name: t.albumName,
      });
      if (items.length >= limit) break;
    }

    return { items, total: offset + items.length + (items.length >= limit ? 1 : 0) };
  },
});

export const listGroups = query({
  args: {
    limit: v.optional(v.number()),
    offset: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const limit = Math.min(args.limit ?? 200, 2000);
    const offset = args.offset ?? 0;

    // Aggregate tracks by artist — we must scan all tracks to build groups,
    // but we only return a page of the result to stay under the 8192 limit.
    const byArtist = new Map<string, { artistId?: any; count: number; providers: Set<string> }>();

    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))) {
      const key = track.artistName ?? "Unknown";
      const existing = byArtist.get(key);
      if (existing) {
        existing.count++;
        existing.providers.add(track.provider);
      } else {
        byArtist.set(key, {
          artistId: track.artistId,
          count: 1,
          providers: new Set([track.provider]),
        });
      }
    }

    const allGroups = Array.from(byArtist.entries()).map(([artistName, data]) => ({
      artist_name: artistName,
      artist_id: data.artistId,
      track_count: data.count,
      providers: [...data.providers],
    }));

    // Sort by track count descending, then paginate
    allGroups.sort((a, b) => b.track_count - a.track_count);
    const page = allGroups.slice(offset, offset + limit);

    return { groups: page, total: allGroups.length };
  },
});

export const tasteGrade = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);

    // Single indexed read from precomputed summary
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) {
      return {
        grade: "pending",
        total_artists: 0,
        total_tracks: 0,
        total_offenses: 0,
        total_severity: 0,
        offender_ratio: 0,
      };
    }

    const totalSeverity = summary.offenders.reduce(
      (sum: number, o: any) => sum + o.severityTotal,
      0,
    );

    return {
      grade: summary.grade,
      total_artists: summary.totalArtists,
      total_tracks: summary.totalTracks,
      total_offenses: summary.flaggedArtistCount,
      total_severity: totalSeverity,
      offender_ratio: summary.offenderRatio,
    };
  },
});

export const listOffenders = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);

    // Single indexed read from precomputed summary
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) {
      return { offenders: [], total: 0 };
    }

    // Already sorted by severity in the pipeline
    const offenders = summary.offenders.map((o: any) => ({
      artist_id: o.artistId as string,
      artist_name: o.artistName,
      offense_count: o.categories.length,
      track_count: o.trackCount,
      severity_total: o.severityTotal,
    }));

    return { offenders, total: offenders.length };
  },
});

export const listPlaylists = query({
  args: {
    provider: v.optional(v.string()),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    // First pass: async-iterate tracks to build lightweight playlist groups
    // (only keep the fields we need, not full documents)
    const tracksQuery = args.provider
      ? ctx.db
          .query("userLibraryTracks")
          .withIndex("by_user_provider", (q) =>
            q.eq("userId", user._id).eq("provider", args.provider!),
          )
      : ctx.db
          .query("userLibraryTracks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id));

    type TrackSlim = {
      artistId: string | undefined;
      artistName: string | undefined;
      lastSyncedAt: string | undefined;
    };
    const groups = new Map<
      string,
      { provider: string; playlistName: string; tracks: TrackSlim[] }
    >();
    const playlistArtistIds = new Set<string>();

    for await (const track of tracksQuery) {
      if (!track.playlistName) continue;
      const key = `${track.provider}::${track.playlistName}`;
      if (!groups.has(key)) {
        groups.set(key, {
          provider: track.provider,
          playlistName: track.playlistName,
          tracks: [],
        });
      }
      const aid = track.artistId as string | undefined;
      groups.get(key)!.tracks.push({
        artistId: aid,
        artistName: track.artistName,
        lastSyncedAt: track.lastSyncedAt,
      });
      if (aid) playlistArtistIds.add(aid);
    }

    // Batch-lookup offending artist IDs
    const offenseResults = await Promise.all(
      [...playlistArtistIds].map(async (artistId) => {
        const row = await ctx.db
          .query("offendingArtistIndex")
          .withIndex("by_artistId", (q) =>
            q.eq("artistId", artistId as any),
          )
          .unique();
        return { artistId, offending: row !== null };
      }),
    );
    const offendingArtistIds = new Set(
      offenseResults.filter((r) => r.offending).map((r) => r.artistId),
    );

    const userBlocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
    const blockedArtistIds = new Set(
      userBlocks.map((b) => b.artistId as string),
    );

    const playlists = Array.from(groups.values()).map((g) => {
      const total = g.tracks.length;
      const artistIds = new Set<string>();
      const flaggedArtists = new Set<string>();
      let flaggedCount = 0;
      let lastSynced = "";

      for (const t of g.tracks) {
        if (t.artistId) artistIds.add(t.artistId);
        if (t.lastSyncedAt && t.lastSyncedAt > lastSynced) {
          lastSynced = t.lastSyncedAt;
        }
        const aid = t.artistId;
        if (aid && (offendingArtistIds.has(aid) || blockedArtistIds.has(aid))) {
          flaggedCount++;
          if (t.artistName) flaggedArtists.add(t.artistName);
        }
      }

      const cleanRatio = total > 0 ? (total - flaggedCount) / total : 1;
      let grade = "A+";
      if (cleanRatio < 0.5) grade = "F";
      else if (cleanRatio < 0.6) grade = "D";
      else if (cleanRatio < 0.7) grade = "C";
      else if (cleanRatio < 0.8) grade = "B";
      else if (cleanRatio < 0.95) grade = "A";

      return {
        provider: g.provider,
        playlist_name: g.playlistName,
        total_tracks: total,
        flagged_tracks: flaggedCount,
        clean_ratio: cleanRatio,
        grade,
        unique_artists: artistIds.size,
        flagged_artists: [...flaggedArtists],
        last_synced: lastSynced,
      };
    });

    const gradeOrder: Record<string, number> = {
      F: 0, D: 1, C: 2, B: 3, A: 4, "A+": 5,
    };
    playlists.sort((a, b) => {
      const ga = gradeOrder[a.grade] ?? 5;
      const gb = gradeOrder[b.grade] ?? 5;
      if (ga !== gb) return ga - gb;
      return a.playlist_name.localeCompare(b.playlist_name);
    });

    return { playlists, total: playlists.length };
  },
});

export const getPlaylistTracks = query({
  args: {
    provider: v.string(),
    playlistName: v.string(),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    // Use async iteration + filter instead of .collect() to reduce memory
    const playlistTracks: Doc<"userLibraryTracks">[] = [];
    for await (const t of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )) {
      if (t.playlistName === args.playlistName) {
        playlistTracks.push(t);
      }
    }

    // Collect unique artist IDs and batch-lookup offending index
    const uniqueArtistIds = new Set<string>();
    for (const t of playlistTracks) {
      if (t.artistId) uniqueArtistIds.add(t.artistId as string);
    }

    const offenseResults = await Promise.all(
      [...uniqueArtistIds].map(async (artistId) => {
        const row = await ctx.db
          .query("offendingArtistIndex")
          .withIndex("by_artistId", (q) =>
            q.eq("artistId", artistId as any),
          )
          .unique();
        return { artistId, offending: row !== null };
      }),
    );
    const offendingArtistIds = new Set(
      offenseResults.filter((r) => r.offending).map((r) => r.artistId),
    );

    const userBlocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
    const blockedArtistIds = new Set(
      userBlocks.map((b) => b.artistId as string),
    );

    const mapped = playlistTracks.map((t, i) => {
      const aid = t.artistId as string | undefined;
      let status: "clean" | "flagged" | "blocked" = "clean";
      if (aid && blockedArtistIds.has(aid)) status = "blocked";
      else if (aid && offendingArtistIds.has(aid)) status = "flagged";

      return {
        id: t._id,
        position: i + 1,
        provider_track_id: t.providerTrackId,
        track_name: t.trackName ?? "Unknown",
        album_name: t.albumName,
        artist_id: t.artistId,
        artist_name: t.artistName ?? "Unknown",
        added_at: t.addedAt,
        status,
      };
    });

    return { tracks: mapped, total: mapped.length };
  },
});

export const importTracks = mutation({
  args: {
    provider: v.string(),
    tracks: v.array(
      v.object({
        providerTrackId: v.string(),
        trackName: v.optional(v.string()),
        albumName: v.optional(v.string()),
        artistName: v.optional(v.string()),
        sourceType: v.optional(v.string()),
        playlistName: v.optional(v.string()),
      }),
    ),
    clearExisting: v.optional(v.boolean()),
  },
  handler: async (ctx: MutationCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    // Optionally clear all existing tracks for this user + provider first
    if (args.clearExisting) {
      const existing = await ctx.db
        .query("userLibraryTracks")
        .withIndex("by_user_provider", (q) =>
          q.eq("userId", user._id).eq("provider", args.provider),
        )
        .collect();

      for (const track of existing) {
        await ctx.db.delete(track._id);
      }
    }

    let imported = 0;

    for (const track of args.tracks) {
      const legacyKey = `runtime:track:${user._id}:${args.provider}:${track.providerTrackId}`;
      const existing = await ctx.db
        .query("userLibraryTracks")
        .withIndex("by_legacyKey", (q) => q.eq("legacyKey", legacyKey))
        .unique();

      if (!existing) {
        await ctx.db.insert("userLibraryTracks", {
          legacyKey,
          userId: user._id,
          provider: args.provider,
          providerTrackId: track.providerTrackId,
          trackName: track.trackName,
          albumName: track.albumName,
          artistName: track.artistName,
          sourceType: track.sourceType,
          playlistName: track.playlistName,
          lastSyncedAt: now,
          metadata: {},
          createdAt: now,
          updatedAt: now,
        });
        imported++;
      }
    }

    return { imported, total: args.tracks.length };
  },
});

/**
 * Internal mutation: delete all userLibraryTracks for a given user + provider.
 * Called once at the start of a full library sync (from scheduled sync actions).
 */
// ---------------------------------------------------------------------------
// Cross-provider dedup (display-layer only — never merges/deletes records)
// ---------------------------------------------------------------------------

/** Strip parenthetical suffixes like "(Deluxe Edition)", "(Remastered)", etc. */
function stripParenSuffixes(s: string): string {
  return s.replace(/\s*\((?:deluxe|remaster(?:ed)?|expanded|bonus|anniversary|live|explicit|clean|mono|stereo|single|ep)\b[^)]*\)/gi, "").trim();
}

/** Normalize a string for dedup grouping: lowercase, strip parens, collapse whitespace. */
function normalizeForDedup(s: string | undefined): string {
  if (!s) return "";
  return stripParenSuffixes(s).toLowerCase().replace(/\s+/g, " ").trim();
}

/** Pick the more-complete value: longest non-empty string wins. */
function pickBestString(a: string | undefined, b: string | undefined): string | undefined {
  if (!a) return b;
  if (!b) return a;
  return a.length >= b.length ? a : b;
}

export const listDeduplicated = query({
  args: {
    limit: v.optional(v.number()),
    offset: v.optional(v.number()),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    // Group by normalized (track, artist) key using async iteration
    const groups = new Map<
      string,
      {
        providers: Set<string>;
        canonical: Doc<"userLibraryTracks">;
        possibleDuplicate: boolean;
      }
    >();

    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))) {
      const normTrack = normalizeForDedup(track.trackName);
      const normArtist = normalizeForDedup(track.artistName);

      // Skip rows with no meaningful identity
      if (!normTrack && !normArtist) continue;

      const key = `${normTrack}||${normArtist}`;
      const existing = groups.get(key);

      if (existing) {
        existing.providers.add(track.provider);
        // Keep the record with the most-complete metadata
        const cur = existing.canonical;
        const betterTrackName = pickBestString(cur.trackName, track.trackName);
        const betterAlbumName = pickBestString(cur.albumName, track.albumName);
        const betterArtistName = pickBestString(cur.artistName, track.artistName);

        if (
          betterTrackName !== cur.trackName ||
          betterAlbumName !== cur.albumName ||
          betterArtistName !== cur.artistName
        ) {
          existing.canonical = {
            ...cur,
            trackName: betterTrackName,
            albumName: betterAlbumName,
            artistName: betterArtistName,
          };
        }

        // Flag as possible duplicate if normalization changed something
        if (
          track.trackName !== existing.canonical.trackName ||
          track.artistName !== existing.canonical.artistName
        ) {
          existing.possibleDuplicate = true;
        }
      } else {
        groups.set(key, {
          providers: new Set([track.provider]),
          canonical: track,
          possibleDuplicate: false,
        });
      }
    }

    const deduplicated = Array.from(groups.values()).map((g) => ({
      id: g.canonical._id,
      track_name: g.canonical.trackName ?? null,
      album_name: g.canonical.albumName ?? null,
      artist_name: g.canonical.artistName ?? null,
      artist_id: g.canonical.artistId ?? null,
      source_type: g.canonical.sourceType ?? null,
      playlist_name: g.canonical.playlistName ?? null,
      added_at: g.canonical.addedAt ?? null,
      last_synced: g.canonical.lastSyncedAt ?? null,
      providers: [...g.providers],
      possible_duplicate: g.possibleDuplicate,
    }));

    // Sort: multi-provider items first, then alphabetically by track name
    deduplicated.sort((a, b) => {
      const provDiff = b.providers.length - a.providers.length;
      if (provDiff !== 0) return provDiff;
      return (a.track_name ?? "").localeCompare(b.track_name ?? "");
    });

    const total = deduplicated.length;
    const offset = args.offset ?? 0;
    const limit = args.limit ?? 50;
    const page = deduplicated.slice(offset, offset + limit);

    return { items: page, total, offset, limit };
  },
});

export const _clearProviderTracks = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
  },
  handler: async (ctx: MutationCtx, args) => {
    // Single-transaction deletion — keeps the operation atomic so subsequent
    // imports don't conflict with ongoing continuation batches.
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .collect();

    for (const track of tracks) {
      await ctx.db.delete(track._id);
    }

    return { deleted: tracks.length };
  },
});
