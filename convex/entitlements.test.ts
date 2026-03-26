/// <reference types="vite/client" />
import { convexTest } from "convex-test";
import { expect, test, describe, beforeEach } from "vitest";
import { internal } from "./_generated/api";
import schema from "./schema";
import {
  getFeatureLimits,
  FREE_LIMITS,
  PRO_LIMITS,
  TEAM_LIMITS,
} from "./lib/entitlements";

const modules = import.meta.glob("./**/*.ts");

describe("getFeatureLimits", () => {
  test("free plan limits are correct", () => {
    const limits = getFeatureLimits("free");
    expect(limits.maxConnections).toBe(1);
    expect(limits.maxScansPerMonth).toBe(1);
    expect(limits.maxCommunityListSubscriptions).toBe(1);
    expect(limits.canCreateCommunityLists).toBe(false);
    expect(limits.canAutoEnforce).toBe(false);
    expect(limits.canExport).toBe(false);
    expect(limits.maxPlaylistGrades).toBe(3);
  });

  test("pro plan has unlimited scans (-1)", () => {
    const limits = getFeatureLimits("pro");
    expect(limits.maxScansPerMonth).toBe(-1);
    expect(limits.maxConnections).toBe(5);
    expect(limits.maxCommunityListSubscriptions).toBe(-1);
    expect(limits.canCreateCommunityLists).toBe(true);
    expect(limits.canAutoEnforce).toBe(true);
    expect(limits.canExport).toBe(true);
    expect(limits.maxPlaylistGrades).toBe(-1);
  });

  test("team plan has 5 seats", () => {
    const limits = getFeatureLimits("team");
    expect(limits.maxSeats).toBe(5);
    // team should also have pro-level feature access
    expect(limits.maxConnections).toBe(5);
    expect(limits.maxScansPerMonth).toBe(-1);
    expect(limits.canCreateCommunityLists).toBe(true);
    expect(limits.canAutoEnforce).toBe(true);
    expect(limits.canExport).toBe(true);
    expect(limits.maxPlaylistGrades).toBe(-1);
  });

  test("unknown plan defaults to free limits", () => {
    const limits = getFeatureLimits("unknown");
    expect(limits).toEqual(FREE_LIMITS);
  });
});

describe("entitlements with database", () => {
  function createTestUser(t: ReturnType<typeof convexTest>) {
    return t.run(async (ctx) => {
      const now = new Date().toISOString();
      const userId = await ctx.db.insert("users", {
        legacyKey: `test:user:${Date.now()}`,
        authSubject: `test|user_${Date.now()}`,
        email: "test@example.com",
        displayName: "Test User",
        createdAt: now,
        updatedAt: now,
      });
      return userId;
    });
  }

  function createSubscription(
    t: ReturnType<typeof convexTest>,
    userId: any,
    overrides: Record<string, any> = {},
  ) {
    return t.run(async (ctx) => {
      const now = new Date().toISOString();
      const subId = await ctx.db.insert("subscriptions", {
        legacyKey: `test:sub:${Date.now()}`,
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_test123",
        stripeSubscriptionId: "sub_test123",
        stripePriceId: "price_test123",
        plan: "pro",
        status: "active",
        currentPeriodStart: now,
        currentPeriodEnd: new Date(
          Date.now() + 30 * 24 * 60 * 60 * 1000,
        ).toISOString(),
        ...overrides,
      });
      return subId;
    });
  }

  test("getUserPlan returns free for users with no subscription", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    const result = await t.run(async (ctx) => {
      const { getUserPlan } = await import("./lib/entitlements");
      return getUserPlan(ctx, userId);
    });

    expect(result.plan).toBe("free");
    expect(result.isActive).toBe(false);
    expect(result.subscription).toBeNull();
  });

  test("getUserPlan returns correct plan for active subscription", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "active" });

    const result = await t.run(async (ctx) => {
      const { getUserPlan } = await import("./lib/entitlements");
      return getUserPlan(ctx, userId);
    });

    expect(result.plan).toBe("pro");
    expect(result.isActive).toBe(true);
    expect(result.subscription).not.toBeNull();
  });

  test("getUserPlan returns free for canceled subscription", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "canceled" });

    const result = await t.run(async (ctx) => {
      const { getUserPlan } = await import("./lib/entitlements");
      return getUserPlan(ctx, userId);
    });

    expect(result.plan).toBe("free");
    expect(result.isActive).toBe(false);
    expect(result.subscription).not.toBeNull();
  });

  test("getUserPlan treats trialing as active", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "trialing" });

    const result = await t.run(async (ctx) => {
      const { getUserPlan } = await import("./lib/entitlements");
      return getUserPlan(ctx, userId);
    });

    expect(result.plan).toBe("pro");
    expect(result.isActive).toBe(true);
  });

  test("requirePlan throws for wrong plan", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    await expect(
      t.run(async (ctx) => {
        const { requirePlan } = await import("./lib/entitlements");
        return requirePlan(ctx, userId, ["pro", "team"]);
      }),
    ).rejects.toThrow();
  });

  test("requirePlan passes for correct plan", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "active" });

    const result = await t.run(async (ctx) => {
      const { requirePlan } = await import("./lib/entitlements");
      return requirePlan(ctx, userId, ["pro", "team"]);
    });

    expect(result.plan).toBe("pro");
  });

  test("checkFeatureAccess gates connections for free plan", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    // Free user with no connections should be allowed
    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "connections");
    });

    expect(result.allowed).toBe(true);
    expect(result.limit).toBe(1);
  });

  test("checkFeatureAccess gates connections when at limit", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    // Add a connection to hit the free limit
    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("providerConnections", {
        legacyKey: `test:conn:${Date.now()}`,
        createdAt: now,
        updatedAt: now,
        userId,
        provider: "spotify",
        status: "active",
      });
    });

    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "connections");
    });

    expect(result.allowed).toBe(false);
    expect(result.reason).toBeDefined();
    expect(result.currentUsage).toBe(1);
    expect(result.limit).toBe(1);
  });

  test("checkFeatureAccess allows autoEnforce for pro users", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "active" });

    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "autoEnforce");
    });

    expect(result.allowed).toBe(true);
  });

  test("checkFeatureAccess denies autoEnforce for free users", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "autoEnforce");
    });

    expect(result.allowed).toBe(false);
    expect(result.reason).toBeDefined();
  });

  test("checkFeatureAccess denies export for free users", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "export");
    });

    expect(result.allowed).toBe(false);
  });

  test("checkFeatureAccess allows export for pro users", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);
    await createSubscription(t, userId, { plan: "pro", status: "active" });

    const result = await t.run(async (ctx) => {
      const { checkFeatureAccess } = await import("./lib/entitlements");
      return checkFeatureAccess(ctx, userId, "export");
    });

    expect(result.allowed).toBe(true);
  });
});
