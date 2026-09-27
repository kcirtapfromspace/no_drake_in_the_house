import { v } from "convex/values";
import { internalMutation } from "./_generated/server";
import { internal } from "./_generated/api";

export const RESEARCH_LEASE_MS = 10 * 60 * 1000;

export function researchCoolingDown(artist: { researchStartedAt?: number }) {
  return artist.researchStartedAt !== undefined && Date.now() - artist.researchStartedAt < RESEARCH_LEASE_MS;
}

/** Claim acquisition separately from the durable per-source evaluation jobs. */
export const start = internalMutation({
  args: { artistId: v.id("artists") },
  handler: async (ctx, { artistId }) => {
    const artist = await ctx.db.get(artistId);
    if (!artist || researchCoolingDown(artist)) return null;
    const generation = (artist.researchGeneration ?? 0) + 1;
    await ctx.db.patch(artistId, { researchGeneration: generation, researchStartedAt: Date.now(),
      investigationStatus: "in_progress", updatedAt: new Date().toISOString() });
    await ctx.scheduler.runAfter(RESEARCH_LEASE_MS, internal.researchLifecycle.expire, { artistId, generation });
    return generation;
  },
});

export const queued = internalMutation({
  args: { artistId: v.id("artists"), generation: v.number() },
  handler: async (ctx, { artistId, generation }) => {
    const artist = await ctx.db.get(artistId);
    // A fast completion callback may arrive before the 202 response.
    if (artist?.researchGeneration === generation && artist.investigationStatus === "in_progress") {
      await ctx.db.patch(artistId, { investigationStatus: "queued", updatedAt: new Date().toISOString() });
    }
  },
});

export const complete = internalMutation({
  args: { artistId: v.id("artists"), generation: v.number(), success: v.boolean() },
  handler: async (ctx, { artistId, generation, success }) => {
    const artist = await ctx.db.get(artistId);
    if (!artist || artist.researchGeneration !== generation ||
        !["in_progress", "queued"].includes(artist.investigationStatus ?? "")) return { accepted: false };
    const now = new Date().toISOString();
    await ctx.db.patch(artistId, { investigationStatus: success ? "completed" : "failed", updatedAt: now,
      ...(success ? { lastInvestigatedAt: now } : {}) });
    return { accepted: true };
  },
});

export const expire = internalMutation({
  args: { artistId: v.id("artists"), generation: v.number() },
  handler: async (ctx, { artistId, generation }) => {
    const artist = await ctx.db.get(artistId);
    if (artist?.researchGeneration === generation && !researchCoolingDown(artist) &&
        ["queued", "in_progress"].includes(artist.investigationStatus ?? "")) {
      await ctx.db.patch(artistId, { investigationStatus: "failed", updatedAt: new Date().toISOString() });
    }
  },
});
