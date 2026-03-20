import { ConvexError, v } from "convex/values";
import type { Doc, Id } from "./_generated/dataModel";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

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
