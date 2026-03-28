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

    const allTracks = await tracksQuery.collect();
    const offset = args.offset ?? 0;
    const limit = args.limit ?? 50;
    const paginated = allTracks.slice(offset, offset + limit);

    return {
      tracks: paginated.map((t) => ({
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
      })),
      total: allTracks.length,
      offset,
      limit,
    };
  },
});

export const listItems = query({
  args: {
    provider: v.optional(v.string()),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const tracks = args.provider
      ? await ctx.db
          .query("userLibraryTracks")
          .withIndex("by_user_provider", (q) =>
            q.eq("userId", user._id).eq("provider", args.provider!),
          )
          .collect()
      : await ctx.db
          .query("userLibraryTracks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect();

    return {
      items: tracks.map((t) => ({
        id: t._id,
        provider: t.provider,
        track_name: t.trackName,
        artist_name: t.artistName,
        album_name: t.albumName,
      })),
      total: tracks.length,
    };
  },
});

export const listGroups = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const byArtist = new Map<string, Doc<"userLibraryTracks">[]>();
    for (const track of tracks) {
      const key = track.artistName ?? "Unknown";
      if (!byArtist.has(key)) byArtist.set(key, []);
      byArtist.get(key)!.push(track);
    }

    const groups = Array.from(byArtist.entries()).map(([artistName, items]) => ({
      artist_name: artistName,
      artist_id: items[0].artistId,
      track_count: items.length,
      providers: [...new Set(items.map((t) => t.provider))],
    }));

    return { groups, total: groups.length };
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

    const tracks = args.provider
      ? await ctx.db
          .query("userLibraryTracks")
          .withIndex("by_user_provider", (q) =>
            q.eq("userId", user._id).eq("provider", args.provider!),
          )
          .collect()
      : await ctx.db
          .query("userLibraryTracks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect();

    // Collect unique artist IDs from playlist tracks, then batch-lookup
    const playlistArtistIds = new Set<string>();
    for (const track of tracks) {
      if (track.playlistName && track.artistId) {
        playlistArtistIds.add(track.artistId as string);
      }
    }

    // Parallel indexed lookups against offendingArtistIndex
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

    const groups = new Map<
      string,
      { provider: string; playlistName: string; tracks: Doc<"userLibraryTracks">[] }
    >();

    for (const track of tracks) {
      if (!track.playlistName) continue;
      const key = `${track.provider}::${track.playlistName}`;
      if (!groups.has(key)) {
        groups.set(key, {
          provider: track.provider,
          playlistName: track.playlistName,
          tracks: [],
        });
      }
      groups.get(key)!.tracks.push(track);
    }

    const playlists = Array.from(groups.values()).map((g) => {
      const total = g.tracks.length;
      const artistIds = new Set<string>();
      const flaggedArtists = new Set<string>();
      let flaggedCount = 0;
      let lastSynced = "";

      for (const t of g.tracks) {
        if (t.artistId) artistIds.add(t.artistId as string);
        if (t.lastSyncedAt && t.lastSyncedAt > lastSynced) {
          lastSynced = t.lastSyncedAt;
        }
        const aid = t.artistId as string | undefined;
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

    // Sort: worst grade first, then alphabetically
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

    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .collect();

    const playlistTracks = tracks.filter(
      (t) => t.playlistName === args.playlistName,
    );

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
export const _clearProviderTracks = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
  },
  handler: async (ctx: MutationCtx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .collect();

    let deleted = 0;
    for (const track of tracks) {
      await ctx.db.delete(track._id);
      deleted++;
    }

    return { deleted };
  },
});
