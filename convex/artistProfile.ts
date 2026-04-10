import { ConvexError, v } from "convex/values";
import type { Doc, Id } from "./_generated/dataModel";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

const severityWeight: Record<string, number> = {
  minor: 1,
  moderate: 3,
  severe: 7,
  egregious: 12,
};

export const getProfile = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const artist = await ctx.db.get(args.artistId);
    if (!artist) {
      return null;
    }

    const [offenses, collaborations, blocks] = await Promise.all([
      ctx.db
        .query("artistOffenses")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId1", (q) => q.eq("artistId1", args.artistId))
        .collect(),
      ctx.db
        .query("userArtistBlocks")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
    ]);

    const troubleScore = offenses.reduce(
      (total, o) => total + (severityWeight[o.severity] ?? 2),
      0,
    );

    return {
      id: artist._id,
      canonical_name: artist.canonicalName,
      external_ids: artist.externalIds ?? {},
      aliases: artist.aliases ?? [],
      metadata: artist.metadata ?? {},
      status: artist.status ?? "active",
      offense_count: offenses.length,
      collaboration_count: collaborations.length,
      block_count: blocks.length,
      trouble_score: troubleScore,
      offenses: offenses.map((o) => ({
        id: o._id,
        category: o.category,
        severity: o.severity,
        title: o.title,
        description: o.description,
        incident_date: o.incidentDate,
        status: o.status,
      })),
      categories: [...new Set(offenses.map((o) => o.category))],
      severity_breakdown: offenses.reduce<Record<string, number>>((acc, o) => {
        acc[o.severity] = (acc[o.severity] ?? 0) + 1;
        return acc;
      }, {}),
    };
  },
});

export const getStreamingMetrics = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const artist = await ctx.db.get(args.artistId);
    if (!artist) {
      return null;
    }

    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const byProvider = new Map<string, number>();
    for (const track of tracks) {
      byProvider.set(track.provider, (byProvider.get(track.provider) ?? 0) + 1);
    }

    return {
      artist_id: args.artistId,
      artist_name: artist.canonicalName,
      total_library_tracks: tracks.length,
      by_provider: Object.fromEntries(byProvider),
      unique_users: new Set(tracks.map((t) => t.userId)).size,
    };
  },
});

export const getCatalog = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    // Get albums for this artist (lightweight — only album metadata)
    const albumLinks = await ctx.db
      .query("albumArtists")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .take(200);

    // Build album-grouped catalog
    const tracks: {
      id: string;
      title: string;
      album: string | null;
      albumCover: string | null;
      role: string;
      year: number | null;
      isBlocked: boolean;
      collaborators: string[];
      duration: string | null;
      providers: string[];
    }[] = [];

    // Fetch tracks per album (much more efficient than scanning all tracks)
    for (const link of albumLinks) {
      const album = await ctx.db.get(link.albumId);
      if (!album) continue;

      const albumMeta = (album.metadata ?? {}) as Record<string, any>;
      const albumCover = albumMeta.coverUrl ?? null;
      const year = album.releaseDate
        ? new Date(album.releaseDate).getFullYear()
        : null;

      const albumTracks = await ctx.db
        .query("tracks")
        .withIndex("by_albumId", (q) => q.eq("albumId", album._id))
        .take(100);

      for (const ct of albumTracks) {
        // Parse featuring artists from track title
        const collaborators: string[] = [];
        const featMatch = ct.title.match(
          /\(?(?:feat\.?|ft\.?|featuring|with)\s+(.+?)\)?$/i,
        );
        if (featMatch) {
          collaborators.push(
            ...featMatch[1].split(/[,&]/).map((s: string) => s.trim()).filter(Boolean),
          );
        }

        tracks.push({
          id: ct._id,
          title: ct.title,
          album: album.title,
          albumCover,
          role: "main",
          year,
          isBlocked: false,
          collaborators,
          duration: ct.duration ? `${Math.floor(ct.duration / 60000)}:${String(Math.floor((ct.duration % 60000) / 1000)).padStart(2, "0")}` : null,
          providers: [],
        });
      }
    }

    // If no albums found, fall back to tracks by artistId (limited)
    if (tracks.length === 0) {
      const directTracks = await ctx.db
        .query("tracks")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .take(200);

      for (const ct of directTracks) {
        let albumTitle: string | null = null;
        let albumCover: string | null = null;
        let year: number | null = null;
        if (ct.albumId) {
          const album = await ctx.db.get(ct.albumId);
          if (album) {
            albumTitle = album.title;
            year = album.releaseDate ? new Date(album.releaseDate).getFullYear() : null;
            albumCover = (album.metadata as Record<string, any>)?.coverUrl ?? null;
          }
        }

        tracks.push({
          id: ct._id,
          title: ct.title,
          album: albumTitle,
          albumCover,
          role: "main",
          year,
          isBlocked: false,
          collaborators: [],
          duration: ct.duration ? `${Math.floor(ct.duration / 60000)}:${String(Math.floor((ct.duration % 60000) / 1000)).padStart(2, "0")}` : null,
          providers: [],
        });
      }
    }

    // Sort by year (newest first), then album name
    tracks.sort((a, b) => {
      const yearCmp = (b.year ?? 0) - (a.year ?? 0);
      return yearCmp !== 0 ? yearCmp : (a.album ?? "").localeCompare(b.album ?? "");
    });

    return {
      artist_id: args.artistId,
      artist_name: artist.canonicalName,
      tracks,
    };
  },
});

export const getCredits = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    // Build credits from userLibraryTracks by parsing featuring artists
    const libraryTracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const userTracks = libraryTracks.filter((t) => t.userId === user._id);

    // Extract collaborators from track names and deduplicate
    const collabMap = new Map<
      string,
      { name: string; trackCount: number; isFlagged: boolean; imageUrl: string | null }
    >();

    for (const t of userTracks) {
      if (!t.trackName) continue;
      // Parse "feat.", "ft.", "featuring", "with", "&" patterns
      const featMatch = t.trackName.match(
        /\(?(?:feat\.?|ft\.?|featuring|with)\s+(.+?)\)?$/i,
      );
      if (!featMatch) continue;

      const names = featMatch[1]
        .split(/[,&]/)
        .map((s) => s.trim())
        .filter((s) => s.length > 0);

      for (const name of names) {
        const key = name.toLowerCase();
        const existing = collabMap.get(key);
        if (existing) {
          existing.trackCount++;
        } else {
          collabMap.set(key, {
            name,
            trackCount: 1,
            isFlagged: false,
            imageUrl: null,
          });
        }
      }
    }

    // Convert to sorted arrays
    const writers = [...collabMap.entries()]
      .map(([key, entry]) => ({
        id: `name:${key}`,
        name: entry.name,
        role: "collaborator",
        track_count: entry.trackCount,
        is_flagged: entry.isFlagged,
        image_url: entry.imageUrl,
      }))
      .sort((a, b) => b.track_count - a.track_count);

    return { writers, producers: [] };
  },
});

export const reportError = mutation({
  args: {
    offenseId: v.id("artistOffenses"),
    reason: v.string(),
    details: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const offense = await ctx.db.get(args.offenseId);
    if (!offense) {
      throw new ConvexError("Offense not found.");
    }

    const metadata = (offense.metadata ?? {}) as Record<string, any>;
    const errorReports = Array.isArray(metadata.errorReports)
      ? metadata.errorReports
      : [];
    errorReports.push({
      userId: user._id,
      reason: args.reason,
      details: args.details,
      reportedAt: nowIso(),
    });

    await ctx.db.patch(offense._id, {
      metadata: { ...metadata, errorReports },
      updatedAt: nowIso(),
    });

    return { success: true };
  },
});

export const batchTrackBlock = mutation({
  args: {
    artistId: v.id("artists"),
    trackIds: v.array(v.id("tracks")),
    reason: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();
    let blocked = 0;

    for (const trackId of args.trackIds) {
      const existing = await ctx.db
        .query("userTrackBlocks")
        .withIndex("by_trackId", (q) => q.eq("trackId", trackId))
        .collect()
        .then((blocks) =>
          blocks.find(
            (b) => b.userId === user._id && b.artistId === args.artistId,
          ),
        );

      if (!existing) {
        await ctx.db.insert("userTrackBlocks", {
          legacyKey: `runtime:tblock:${user._id}:${trackId}`,
          userId: user._id,
          artistId: args.artistId,
          trackId,
          reason: args.reason,
          metadata: {},
          createdAt: now,
          updatedAt: now,
        });
        blocked++;
      }
    }

    return { blocked, total: args.trackIds.length };
  },
});

export const blockTrack = mutation({
  args: {
    artistId: v.id("artists"),
    trackId: v.optional(v.id("tracks")),
    reason: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const blockId = await ctx.db.insert("userTrackBlocks", {
      legacyKey: `runtime:tblock:${user._id}:${args.trackId ?? "none"}:${Date.now()}`,
      userId: user._id,
      artistId: args.artistId,
      trackId: args.trackId,
      reason: args.reason,
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });

    return await ctx.db.get(blockId);
  },
});

export const unblockTrack = mutation({
  args: {
    trackId: v.id("tracks"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("userTrackBlocks")
      .withIndex("by_trackId", (q) => q.eq("trackId", args.trackId))
      .collect()
      .then((blocks) => blocks.find((b) => b.userId === user._id));

    if (existing) {
      await ctx.db.delete(existing._id);
    }

    return { success: true };
  },
});
