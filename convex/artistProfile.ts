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
