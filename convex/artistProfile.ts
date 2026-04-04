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

    // Get albums linked to this artist
    const albumArtistLinks = await ctx.db
      .query("albumArtists")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    // Get user's track blocks for this artist
    const trackBlocks = await ctx.db
      .query("userTrackBlocks")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();
    const blockedTrackIds = new Set(
      trackBlocks
        .filter((b) => b.userId === user._id)
        .map((b) => b.trackId as string),
    );

    // Fetch albums and their tracks
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
    }[] = [];

    for (const link of albumArtistLinks) {
      const album = await ctx.db.get(link.albumId);
      if (!album) continue;

      const albumTracks = await ctx.db
        .query("tracks")
        .withIndex("by_albumId", (q) => q.eq("albumId", album._id))
        .collect();

      const year = album.releaseDate
        ? new Date(album.releaseDate).getFullYear()
        : null;
      const cover =
        (album.metadata as Record<string, any>)?.coverUrl ?? null;

      for (const track of albumTracks) {
        // Determine role from trackCredits
        const credits = await ctx.db
          .query("trackCredits")
          .withIndex("by_trackId", (q) => q.eq("trackId", track._id))
          .collect();

        const artistCredit = credits.find(
          (c) => c.artistId === args.artistId,
        );
        const role = artistCredit?.role ?? "main";

        // Collaborators are other credited artists on this track
        const collabNames = credits
          .filter((c) => c.artistId !== args.artistId)
          .map((c) => c.creditedName);

        const meta = track.metadata as Record<string, any> | undefined;
        tracks.push({
          id: track._id,
          title: track.title,
          album: album.title,
          albumCover: cover,
          role,
          year,
          isBlocked: blockedTrackIds.has(track._id),
          collaborators: collabNames,
          duration: meta?.duration ?? null,
        });
      }
    }

    // Also get tracks where this artist has credits but isn't the album artist
    const creditLinks = await ctx.db
      .query("trackCredits")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const existingTrackIds = new Set(tracks.map((t) => t.id));
    for (const credit of creditLinks) {
      if (existingTrackIds.has(credit.trackId as string)) continue;

      const track = await ctx.db.get(credit.trackId);
      if (!track) continue;

      let albumTitle: string | null = null;
      let albumYear: number | null = null;
      let albumCover: string | null = null;
      if (track.albumId) {
        const album = await ctx.db.get(track.albumId);
        if (album) {
          albumTitle = album.title;
          albumYear = album.releaseDate
            ? new Date(album.releaseDate).getFullYear()
            : null;
          albumCover =
            (album.metadata as Record<string, any>)?.coverUrl ?? null;
        }
      }

      const otherCredits = await ctx.db
        .query("trackCredits")
        .withIndex("by_trackId", (q) => q.eq("trackId", track._id))
        .collect();

      tracks.push({
        id: track._id,
        title: track.title,
        album: albumTitle,
        albumCover,
        role: credit.role,
        year: albumYear,
        isBlocked: blockedTrackIds.has(track._id),
        collaborators: otherCredits
          .filter((c) => c.artistId !== args.artistId)
          .map((c) => c.creditedName),
        duration: (track.metadata as Record<string, any>)?.duration ?? null,
      });
    }

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
    await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    // Get all tracks where this artist has credits
    const creditLinks = await ctx.db
      .query("trackCredits")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    // For each track, find other credited people
    const creditMap = new Map<
      string,
      { name: string; role: string; trackCount: number; isFlagged: boolean; imageUrl: string | null }
    >();

    for (const credit of creditLinks) {
      const otherCredits = await ctx.db
        .query("trackCredits")
        .withIndex("by_trackId", (q) => q.eq("trackId", credit.trackId))
        .collect();

      for (const other of otherCredits) {
        if (other.artistId === args.artistId) continue;

        const key = other.artistId
          ? (other.artistId as string)
          : `name:${other.creditedName}`;
        const existing = creditMap.get(key);
        if (existing) {
          existing.trackCount++;
        } else {
          let isFlagged = false;
          let imageUrl: string | null = null;
          if (other.artistId) {
            const otherArtist = await ctx.db.get(other.artistId);
            if (otherArtist) {
              isFlagged = otherArtist.status === "flagged";
              imageUrl =
                (otherArtist.metadata as Record<string, any>)?.imageUrl ??
                null;
            }
          }
          creditMap.set(key, {
            name: other.creditedName,
            role: other.role,
            trackCount: 1,
            isFlagged,
            imageUrl,
          });
        }
      }
    }

    const writers: {
      id: string;
      name: string;
      role: string;
      track_count: number;
      is_flagged: boolean;
      image_url: string | null;
    }[] = [];
    const producers: typeof writers = [];

    for (const [id, entry] of creditMap) {
      const item = {
        id,
        name: entry.name,
        role: entry.role,
        track_count: entry.trackCount,
        is_flagged: entry.isFlagged,
        image_url: entry.imageUrl,
      };
      if (entry.role === "producer") {
        producers.push(item);
      } else {
        writers.push(item);
      }
    }

    writers.sort((a, b) => b.track_count - a.track_count);
    producers.sort((a, b) => b.track_count - a.track_count);

    return { writers, producers };
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
