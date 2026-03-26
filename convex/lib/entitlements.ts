import { ConvexError } from "convex/values";
import type { QueryCtx } from "../_generated/server";
import type { Doc, Id } from "../_generated/dataModel";

export type PlanType = "free" | "pro" | "team";

export interface FeatureLimits {
  maxConnections: number;
  maxScansPerMonth: number;
  maxCommunityListSubscriptions: number;
  canCreateCommunityLists: boolean;
  canAutoEnforce: boolean;
  canExport: boolean;
  maxPlaylistGrades: number;
  maxSeats?: number;
}

export const FREE_LIMITS: FeatureLimits = {
  maxConnections: 1,
  maxScansPerMonth: 1,
  maxCommunityListSubscriptions: 1,
  canCreateCommunityLists: false,
  canAutoEnforce: false,
  canExport: false,
  maxPlaylistGrades: 3,
};

export const PRO_LIMITS: FeatureLimits = {
  maxConnections: 5,
  maxScansPerMonth: -1,
  maxCommunityListSubscriptions: -1,
  canCreateCommunityLists: true,
  canAutoEnforce: true,
  canExport: true,
  maxPlaylistGrades: -1,
};

export const TEAM_LIMITS: FeatureLimits = {
  maxConnections: 5,
  maxScansPerMonth: -1,
  maxCommunityListSubscriptions: -1,
  canCreateCommunityLists: true,
  canAutoEnforce: true,
  canExport: true,
  maxPlaylistGrades: -1,
  maxSeats: 5,
};

const ACTIVE_STATUSES = ["active", "trialing", "past_due"];

/**
 * Returns the feature limits for a given plan.
 * Unknown plans default to free limits.
 */
export function getFeatureLimits(plan: string): FeatureLimits {
  switch (plan) {
    case "pro":
      return { ...PRO_LIMITS };
    case "team":
      return { ...TEAM_LIMITS };
    case "free":
    default:
      return { ...FREE_LIMITS };
  }
}

/**
 * Gets the current user's plan and subscription status.
 */
export async function getUserPlan(
  ctx: QueryCtx,
  userId: Id<"users">,
): Promise<{
  plan: PlanType;
  isActive: boolean;
  subscription: Doc<"subscriptions"> | null;
}> {
  const subscription = await ctx.db
    .query("subscriptions")
    .withIndex("by_userId", (q) => q.eq("userId", userId))
    .order("desc")
    .first();

  if (!subscription) {
    return { plan: "free", isActive: false, subscription: null };
  }

  const isActive = ACTIVE_STATUSES.includes(subscription.status);

  return {
    plan: isActive ? subscription.plan : "free",
    isActive,
    subscription,
  };
}

/**
 * Throws a ConvexError if the user doesn't have one of the required plans.
 */
export async function requirePlan(
  ctx: QueryCtx,
  userId: Id<"users">,
  requiredPlans: string[],
): Promise<{ plan: PlanType; isActive: boolean; subscription: Doc<"subscriptions"> | null }> {
  const userPlan = await getUserPlan(ctx, userId);

  if (!requiredPlans.includes(userPlan.plan)) {
    throw new ConvexError(
      `This feature requires one of the following plans: ${requiredPlans.join(", ")}. Your current plan is: ${userPlan.plan}.`,
    );
  }

  return userPlan;
}

/**
 * Checks whether a user has access to a specific feature.
 * Returns an object indicating whether access is allowed and relevant usage data.
 */
export async function checkFeatureAccess(
  ctx: QueryCtx,
  userId: Id<"users">,
  feature: string,
): Promise<{
  allowed: boolean;
  reason?: string;
  currentUsage?: number;
  limit?: number;
}> {
  const { plan } = await getUserPlan(ctx, userId);
  const limits = getFeatureLimits(plan);

  switch (feature) {
    case "connections": {
      const connections = await ctx.db
        .query("providerConnections")
        .withIndex("by_userId", (q) => q.eq("userId", userId))
        .collect();
      const activeConnections = connections.filter(
        (c) => c.status === "active",
      );
      const currentUsage = activeConnections.length;
      const limit = limits.maxConnections;
      // -1 means unlimited
      if (limit !== -1 && currentUsage >= limit) {
        return {
          allowed: false,
          reason: `You have reached the maximum number of connections (${limit}) for the ${plan} plan. Upgrade to add more.`,
          currentUsage,
          limit,
        };
      }
      return { allowed: true, currentUsage, limit };
    }

    case "scans": {
      const limit = limits.maxScansPerMonth;
      if (limit === -1) {
        return { allowed: true, limit };
      }
      // Count scans in the current month
      const now = new Date();
      const startOfMonth = new Date(
        now.getFullYear(),
        now.getMonth(),
        1,
      ).toISOString();
      const scans = await ctx.db
        .query("libraryScans")
        .withIndex("by_userId", (q) => q.eq("userId", userId))
        .collect();
      const monthlyScans = scans.filter(
        (s) => s.scanStartedAt >= startOfMonth,
      );
      const currentUsage = monthlyScans.length;
      if (currentUsage >= limit) {
        return {
          allowed: false,
          reason: `You have reached the maximum number of scans (${limit}) for the ${plan} plan this month. Upgrade to scan more.`,
          currentUsage,
          limit,
        };
      }
      return { allowed: true, currentUsage, limit };
    }

    case "communityListSubscriptions": {
      const limit = limits.maxCommunityListSubscriptions;
      if (limit === -1) {
        return { allowed: true, limit };
      }
      const subs = await ctx.db
        .query("userListSubscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", userId))
        .collect();
      const currentUsage = subs.length;
      if (currentUsage >= limit) {
        return {
          allowed: false,
          reason: `You have reached the maximum number of community list subscriptions (${limit}) for the ${plan} plan. Upgrade to subscribe to more lists.`,
          currentUsage,
          limit,
        };
      }
      return { allowed: true, currentUsage, limit };
    }

    case "createCommunityLists": {
      if (!limits.canCreateCommunityLists) {
        return {
          allowed: false,
          reason: `Creating community lists is not available on the ${plan} plan. Upgrade to Pro or Team to create community lists.`,
        };
      }
      return { allowed: true };
    }

    case "autoEnforce": {
      if (!limits.canAutoEnforce) {
        return {
          allowed: false,
          reason: `Auto-enforcement is not available on the ${plan} plan. Upgrade to Pro or Team to enable auto-enforcement.`,
        };
      }
      return { allowed: true };
    }

    case "export": {
      if (!limits.canExport) {
        return {
          allowed: false,
          reason: `Export is not available on the ${plan} plan. Upgrade to Pro or Team to export data.`,
        };
      }
      return { allowed: true };
    }

    case "playlistGrades": {
      const limit = limits.maxPlaylistGrades;
      if (limit === -1) {
        return { allowed: true, limit };
      }
      // For playlist grades we just return the limit; caller checks usage
      return { allowed: true, limit };
    }

    default:
      return {
        allowed: false,
        reason: `Unknown feature: ${feature}`,
      };
  }
}
