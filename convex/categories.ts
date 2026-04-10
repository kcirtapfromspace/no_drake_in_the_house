import { v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const CATEGORY_COPY: Record<string, { name: string; description: string }> = {
  domestic_violence: {
    name: "Domestic Violence",
    description: "Artists flagged for documented domestic violence incidents.",
  },
  sexual_misconduct: {
    name: "Sexual Misconduct",
    description: "Artists tied to sexual misconduct allegations or findings.",
  },
  sexual_assault: {
    name: "Sexual Assault",
    description: "Artists associated with sexual assault cases or convictions.",
  },
  child_abuse: {
    name: "Child Abuse",
    description: "Artists linked to child abuse or exploitation cases.",
  },
  hate_speech: {
    name: "Hate Speech",
    description: "Artists with hate speech or extremist rhetoric evidence.",
  },
  racism: {
    name: "Racism",
    description: "Artists with racist incidents or persistent racist conduct.",
  },
  antisemitism: {
    name: "Antisemitism",
    description: "Artists associated with antisemitic acts or statements.",
  },
  homophobia: {
    name: "Homophobia",
    description: "Artists with documented homophobic speech or conduct.",
  },
  violent_crime: {
    name: "Violent Crime",
    description: "Artists tied to assaults, shootings, or other violent crimes.",
  },
  drug_trafficking: {
    name: "Drug Trafficking",
    description: "Artists connected to trafficking or major narcotics cases.",
  },
  fraud: {
    name: "Fraud",
    description: "Artists associated with fraud or financial deception cases.",
  },
  harassment: {
    name: "Harassment",
    description: "Artists with documented harassment or stalking incidents.",
  },
  animal_cruelty: {
    name: "Animal Cruelty",
    description: "Artists linked to animal cruelty or abuse cases.",
  },
  certified_creeper: {
    name: "Certified Creeper",
    description: "High-signal artists with repeated predatory behavior evidence.",
  },
};

/** Map research pipeline category names to canonical CATEGORY_COPY keys. */
const CATEGORY_ALIASES: Record<string, string> = {
  violent_crimes: "violent_crime",
  financial_crimes: "fraud",
  drug_offenses: "drug_trafficking",
  plagiarism: "fraud",
};

function normalizeCategoryKey(raw: string): string {
  const key = raw.toLowerCase().replace(/[\s-]+/g, "_");
  return CATEGORY_ALIASES[key] ?? key;
}

function categoryCopy(category: string) {
  return (
    CATEGORY_COPY[category] ?? {
      name: category
        .split("_")
        .map((chunk) => chunk.charAt(0).toUpperCase() + chunk.slice(1))
        .join(" "),
      description: "Evidence-backed artist category imported from legacy policy.",
    }
  );
}

export const list = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [subscriptions, offenses] = await Promise.all([
      ctx.db
        .query("categorySubscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db.query("artistOffenses").take(2000),
    ]);

    const subscribed = new Set(subscriptions.map((subscription) => subscription.category));

    const artistSets = new Map<string, Set<string>>();
    const offenseCounts = new Map<string, number>();

    for (const offense of offenses) {
      const category = normalizeCategoryKey(offense.category);
      offenseCounts.set(category, (offenseCounts.get(category) ?? 0) + 1);
      if (!artistSets.has(category)) artistSets.set(category, new Set());
      artistSets.get(category)!.add(offense.artistId as string);
    }

    const allCategories = Object.keys(CATEGORY_COPY);

    return allCategories
      .map((category) => ({
        id: category,
        name: categoryCopy(category).name,
        description: categoryCopy(category).description,
        artist_count: artistSets.get(category)?.size ?? 0,
        offense_count: offenseCounts.get(category) ?? 0,
        subscribed: subscribed.has(category),
      }))
      .sort((a, b) => b.offense_count - a.offense_count);
  },
});

export const subscribe = mutation({
  args: {
    category: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("categorySubscriptions")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((entries) =>
        entries.find((entry) => entry.category === args.category) ?? null,
      );

    if (!existing) {
      await ctx.db.insert("categorySubscriptions", {
        legacyKey: `runtime:category:${user._id}:${args.category}`,
        userId: user._id,
        category: args.category,
        createdAt: nowIso(),
        updatedAt: nowIso(),
      });
    }

    return { success: true };
  },
});

export const unsubscribe = mutation({
  args: {
    category: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const existing = await ctx.db
      .query("categorySubscriptions")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((entries) =>
        entries.find((entry) => entry.category === args.category) ?? null,
      );

    if (existing) {
      await ctx.db.delete(existing._id);
    }

    return { success: true };
  },
});

export const blockedArtists = query({
  args: {
    category: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    let categories = args.category ? [args.category] : [];

    if (categories.length === 0) {
      const subscriptions = await ctx.db
        .query("categorySubscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect();
      categories = subscriptions.map((subscription) => subscription.category);
    }

    if (categories.length === 0) {
      return [];
    }

    const offenses = await ctx.db.query("artistOffenses").take(2000);
    const matching = offenses.filter((offense) =>
      categories.includes(normalizeCategoryKey(offense.category)),
    );

    const byArtist = new Map<string, typeof matching[number]>();
    for (const offense of matching) {
      const current = byArtist.get(offense.artistId);
      if (!current || current.severity < offense.severity) {
        byArtist.set(offense.artistId, offense);
      }
    }

    const artists = await Promise.all(
      Array.from(byArtist.values()).map(async (offense) => {
        const artist = await ctx.db.get(offense.artistId);
        if (!artist) {
          return null;
        }
        return {
          id: artist._id,
          name: artist.canonicalName,
          category: offense.category,
          severity: offense.severity,
        };
      }),
    );

    return artists.filter(Boolean);
  },
});
