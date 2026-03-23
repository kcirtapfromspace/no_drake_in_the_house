import { ConvexError, v } from "convex/values";
import type { Doc } from "./_generated/dataModel";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const list = query({
  args: {},
  handler: async (ctx) => {
    const lists = await ctx.db.query("communityLists").collect();

    const withCounts = await Promise.all(
      lists.map(async (list) => {
        const items = await ctx.db
          .query("communityListItems")
          .withIndex("by_listId", (q) => q.eq("listId", list._id))
          .collect();

        const subscriptions = await ctx.db
          .query("userListSubscriptions")
          .collect();
        const subCount = subscriptions.filter(
          (s) => s.listId === list._id,
        ).length;

        return {
          id: list._id,
          owner_user_id: list.ownerUserId ?? "",
          name: list.name,
          description: list.description ?? "",
          criteria: list.criteria ?? "",
          governance_url: list.governanceUrl,
          update_cadence: list.updateCadence ?? "manual",
          version: list.version ?? 1,
          visibility: list.visibility ?? "public",
          created_at: list.createdAt,
          updated_at: list.updatedAt,
          artist_count: items.length,
          subscriber_count: subCount,
        };
      }),
    );

    return withCounts;
  },
});

export const listArtists = query({
  args: {
    listId: v.id("communityLists"),
  },
  handler: async (ctx, args) => {
    const communityList = await ctx.db.get(args.listId);
    if (!communityList) {
      throw new ConvexError("Community list not found.");
    }

    const items = await ctx.db
      .query("communityListItems")
      .withIndex("by_listId", (q) => q.eq("listId", args.listId))
      .collect();

    const artists = await Promise.all(
      items.map(async (item) => {
        const artist = await ctx.db.get(item.artistId);
        return {
          artist: artist
            ? {
                id: artist._id,
                canonical_name: artist.canonicalName,
                external_ids: artist.externalIds ?? {},
                metadata: artist.metadata ?? {},
              }
            : null,
          rationale_link: item.rationaleLink,
          added_at: item.createdAt,
        };
      }),
    );

    const subscriptions = await ctx.db.query("userListSubscriptions").collect();
    const subCount = subscriptions.filter(
      (s) => s.listId === args.listId,
    ).length;

    return {
      id: communityList._id,
      owner_user_id: communityList.ownerUserId ?? "",
      name: communityList.name,
      description: communityList.description ?? "",
      criteria: communityList.criteria ?? "",
      governance_url: communityList.governanceUrl,
      update_cadence: communityList.updateCadence ?? "manual",
      version: communityList.version ?? 1,
      visibility: communityList.visibility ?? "public",
      created_at: communityList.createdAt,
      updated_at: communityList.updatedAt,
      artist_count: items.length,
      subscriber_count: subCount,
      artists: artists.filter((a) => a.artist !== null),
    };
  },
});

export const subscriptions = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const subs = await ctx.db
      .query("userListSubscriptions")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const withLists = await Promise.all(
      subs.map(async (sub) => {
        const communityList = await ctx.db.get(sub.listId);
        if (!communityList) return null;

        const items = await ctx.db
          .query("communityListItems")
          .withIndex("by_listId", (q) => q.eq("listId", sub.listId))
          .collect();

        return {
          list_id: sub.listId,
          list: {
            id: communityList._id,
            owner_user_id: communityList.ownerUserId ?? "",
            name: communityList.name,
            description: communityList.description ?? "",
            criteria: communityList.criteria ?? "",
            governance_url: communityList.governanceUrl,
            update_cadence: communityList.updateCadence ?? "manual",
            version: communityList.version ?? 1,
            visibility: communityList.visibility ?? "public",
            created_at: communityList.createdAt,
            updated_at: communityList.updatedAt,
            artist_count: items.length,
          },
          version_pinned: sub.versionPinned,
          auto_update: sub.autoUpdate ?? true,
          created_at: sub.createdAt,
        };
      }),
    );

    return withLists.filter(Boolean);
  },
});

export const impact = query({
  args: {
    listId: v.id("communityLists"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const communityList = await ctx.db.get(args.listId);
    if (!communityList) {
      throw new ConvexError("Community list not found.");
    }

    const listItems = await ctx.db
      .query("communityListItems")
      .withIndex("by_listId", (q) => q.eq("listId", args.listId))
      .collect();

    const userBlocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const blockedArtistIds = new Set(userBlocks.map((b) => b.artistId));

    let toAdd = 0;
    let toRemove = 0;
    const previewArtists: Array<{ artist: any; action: "add" | "remove" }> = [];

    for (const item of listItems) {
      const artist = await ctx.db.get(item.artistId);
      if (!artist) continue;

      const mapped = {
        id: artist._id,
        canonical_name: artist.canonicalName,
      };

      if (!blockedArtistIds.has(item.artistId)) {
        toAdd++;
        if (previewArtists.length < 10) {
          previewArtists.push({ artist: mapped, action: "add" });
        }
      }
    }

    return {
      list_id: args.listId,
      list_name: communityList.name,
      artists_to_add: toAdd,
      artists_to_remove: toRemove,
      preview_artists: previewArtists,
    };
  },
});

export const subscribe = mutation({
  args: {
    listId: v.id("communityLists"),
    versionPinned: v.optional(v.number()),
    autoUpdate: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const communityList = await ctx.db.get(args.listId);
    if (!communityList) {
      throw new ConvexError("Community list not found.");
    }

    const existing = await ctx.db
      .query("userListSubscriptions")
      .withIndex("by_user_list", (q) =>
        q.eq("userId", user._id).eq("listId", args.listId),
      )
      .unique();

    if (existing) {
      return { success: true, message: "Already subscribed." };
    }

    await ctx.db.insert("userListSubscriptions", {
      legacyKey: `runtime:sub:${user._id}:${args.listId}`,
      userId: user._id,
      listId: args.listId,
      versionPinned: args.versionPinned,
      autoUpdate: args.autoUpdate ?? true,
      createdAt: nowIso(),
      updatedAt: nowIso(),
    });

    return { success: true };
  },
});

export const unsubscribe = mutation({
  args: {
    listId: v.id("communityLists"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("userListSubscriptions")
      .withIndex("by_user_list", (q) =>
        q.eq("userId", user._id).eq("listId", args.listId),
      )
      .unique();

    if (existing) {
      await ctx.db.delete(existing._id);
    }

    return { success: true };
  },
});

export const updateSubscription = mutation({
  args: {
    listId: v.id("communityLists"),
    versionPinned: v.optional(v.number()),
    autoUpdate: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("userListSubscriptions")
      .withIndex("by_user_list", (q) =>
        q.eq("userId", user._id).eq("listId", args.listId),
      )
      .unique();

    if (!existing) {
      throw new ConvexError("Subscription not found.");
    }

    await ctx.db.patch(existing._id, {
      versionPinned: args.versionPinned,
      autoUpdate: args.autoUpdate,
      updatedAt: nowIso(),
    });

    return { success: true };
  },
});

export const create = mutation({
  args: {
    name: v.string(),
    description: v.optional(v.string()),
    criteria: v.optional(v.string()),
    governanceUrl: v.optional(v.string()),
    updateCadence: v.optional(v.string()),
    visibility: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const listId = await ctx.db.insert("communityLists", {
      legacyKey: `runtime:community:${user._id}:${Date.now()}`,
      ownerUserId: user._id,
      name: args.name,
      description: args.description ?? "",
      criteria: args.criteria ?? "",
      governanceUrl: args.governanceUrl,
      updateCadence: args.updateCadence ?? "manual",
      version: 1,
      visibility: args.visibility ?? "public",
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });

    return await ctx.db.get(listId);
  },
});
