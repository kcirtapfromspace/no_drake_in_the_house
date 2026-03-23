import { v } from "convex/values";
import { action, mutation, query } from "./_generated/server";
import { requireCurrentUser } from "./lib/auth";

const _computeGrade = query({
  args: {
    provider: v.string(),
    playlistId: v.string(),
  },
  handler: async (ctx, args) => {
    const allOffenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIds = new Set(
      allOffenses.map((o) => o.artistId as string),
    );

    const allTracks = await ctx.db.query("userLibraryTracks").collect();
    const tracks = allTracks.filter(
      (t) =>
        t.provider === args.provider && t.playlistName === args.playlistId,
    );

    const totalTracks = tracks.length;
    const flagged = tracks.filter(
      (t) => t.artistId && offendingArtistIds.has(t.artistId as string),
    );

    const ratio =
      totalTracks > 0 ? (totalTracks - flagged.length) / totalTracks : 1;
    let grade = "A+";
    if (ratio < 0.5) grade = "F";
    else if (ratio < 0.6) grade = "D";
    else if (ratio < 0.7) grade = "C";
    else if (ratio < 0.8) grade = "B";
    else if (ratio < 0.95) grade = "A";

    return {
      playlist_id: args.playlistId,
      provider: args.provider,
      grade,
      total_tracks: totalTracks,
      flagged_tracks: flagged.length,
      clean_ratio: ratio,
      flagged_artists: [
        ...new Set(flagged.map((t) => t.artistName).filter(Boolean)),
      ],
    };
  },
});

export { _computeGrade as computeGrade };

export const gradePlaylist = action({
  args: {
    provider: v.string(),
    playlistId: v.string(),
  },
  handler: async (ctx, args) => {
    const grade = await ctx.runQuery(
      "sanitizer:computeGrade" as any,
      { provider: args.provider, playlistId: args.playlistId },
    );
    return grade;
  },
});

export const suggestReplacements = action({
  args: {
    provider: v.string(),
    playlistId: v.string(),
    flaggedTrackIds: v.optional(v.array(v.string())),
  },
  handler: async (_ctx, args) => {
    return {
      playlist_id: args.playlistId,
      provider: args.provider,
      suggestions: [],
      message:
        "Replacement suggestions will be available once provider API integration is complete.",
    };
  },
});

export const updatePlan = mutation({
  args: {
    planId: v.string(),
    acceptedReplacements: v.optional(v.array(v.string())),
    rejectedReplacements: v.optional(v.array(v.string())),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    return {
      plan_id: args.planId,
      accepted: args.acceptedReplacements?.length ?? 0,
      rejected: args.rejectedReplacements?.length ?? 0,
      status: "updated",
    };
  },
});

export const publishPlan = action({
  args: {
    planId: v.string(),
  },
  handler: async (_ctx, args) => {
    return {
      plan_id: args.planId,
      status: "published",
      message:
        "Plan published. Changes will be applied once provider API integration is complete.",
    };
  },
});
