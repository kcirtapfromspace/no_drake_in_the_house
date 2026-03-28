/// <reference types="vite/client" />
import { convexTest } from "convex-test";
import { expect, test, describe } from "vitest";
import { api, internal } from "./_generated/api";
import schema from "./schema";

const modules = import.meta.glob("./**/*.ts");

// Helper to create a test user directly in the DB
function createTestUser(
  t: ReturnType<typeof convexTest>,
  authSubject = "test-issuer|user_test123",
) {
  return t.run(async (ctx) => {
    const now = new Date().toISOString();
    const userId = await ctx.db.insert("users", {
      legacyKey: `test:user:${Date.now()}:${Math.random()}`,
      authSubject,
      email: "test@example.com",
      displayName: "Test User",
      createdAt: now,
      updatedAt: now,
    });
    return userId;
  });
}

describe("schema: subscriptions table", () => {
  test("can create and query subscription records", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    const subId = await t.run(async (ctx) => {
      const now = new Date().toISOString();
      return ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:1",
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
      });
    });

    const sub = await t.run(async (ctx) => {
      return ctx.db.get(subId);
    });

    expect(sub).not.toBeNull();
    expect(sub!.plan).toBe("pro");
    expect(sub!.status).toBe("active");
    expect(sub!.stripeCustomerId).toBe("cus_test123");
  });

  test("can query subscriptions by userId index", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:idx1",
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_idx1",
        stripeSubscriptionId: "sub_idx1",
        stripePriceId: "price_idx1",
        plan: "pro",
        status: "active",
        currentPeriodStart: now,
        currentPeriodEnd: now,
      });
    });

    const subs = await t.run(async (ctx) => {
      return ctx.db
        .query("subscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", userId))
        .collect();
    });

    expect(subs).toHaveLength(1);
    expect(subs[0].stripeCustomerId).toBe("cus_idx1");
  });

  test("can query subscriptions by stripeSubscriptionId index", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:stripe1",
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_stripe1",
        stripeSubscriptionId: "sub_unique_stripe",
        stripePriceId: "price_stripe1",
        plan: "team",
        status: "trialing",
        currentPeriodStart: now,
        currentPeriodEnd: now,
      });
    });

    const subs = await t.run(async (ctx) => {
      return ctx.db
        .query("subscriptions")
        .withIndex("by_stripeSubscriptionId", (q) =>
          q.eq("stripeSubscriptionId", "sub_unique_stripe"),
        )
        .collect();
    });

    expect(subs).toHaveLength(1);
    expect(subs[0].plan).toBe("team");
  });
});

describe("schema: billingEvents table", () => {
  test("can create and query billing event records", async () => {
    const t = convexTest(schema, modules);

    const eventId = await t.run(async (ctx) => {
      const now = new Date().toISOString();
      return ctx.db.insert("billingEvents", {
        legacyKey: "test:event:1",
        createdAt: now,
        updatedAt: now,
        stripeEventId: "evt_test123",
        eventType: "customer.subscription.created",
        payload: { foo: "bar" },
        processedAt: now,
      });
    });

    const event = await t.run(async (ctx) => {
      return ctx.db.get(eventId);
    });

    expect(event).not.toBeNull();
    expect(event!.stripeEventId).toBe("evt_test123");
    expect(event!.eventType).toBe("customer.subscription.created");
  });

  test("can query billing events by stripeEventId index", async () => {
    const t = convexTest(schema, modules);

    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("billingEvents", {
        legacyKey: "test:event:idx1",
        createdAt: now,
        updatedAt: now,
        stripeEventId: "evt_unique",
        eventType: "invoice.paid",
        payload: {},
      });
    });

    const events = await t.run(async (ctx) => {
      return ctx.db
        .query("billingEvents")
        .withIndex("by_stripeEventId", (q) =>
          q.eq("stripeEventId", "evt_unique"),
        )
        .collect();
    });

    expect(events).toHaveLength(1);
    expect(events[0].eventType).toBe("invoice.paid");
  });
});

describe("syncSubscription mutation", () => {
  test("creates new subscription correctly", async () => {
    const t = convexTest(schema, modules);

    await t.mutation(internal.subscriptions.syncSubscription, {
      stripeCustomerId: "cus_new123",
      stripeSubscriptionId: "sub_new123",
      plan: "pro",
      status: "active",
      currentPeriodStart: new Date().toISOString(),
      currentPeriodEnd: new Date(
        Date.now() + 30 * 24 * 60 * 60 * 1000,
      ).toISOString(),
      stripePriceId: "price_pro_monthly",
      cancelAtPeriodEnd: false,
    });

    const subs = await t.run(async (ctx) => {
      return ctx.db
        .query("subscriptions")
        .withIndex("by_stripeSubscriptionId", (q) =>
          q.eq("stripeSubscriptionId", "sub_new123"),
        )
        .collect();
    });

    expect(subs).toHaveLength(1);
    expect(subs[0].plan).toBe("pro");
    expect(subs[0].status).toBe("active");
    expect(subs[0].stripeCustomerId).toBe("cus_new123");
  });

  test("updates existing subscription", async () => {
    const t = convexTest(schema, modules);

    // Create initial subscription
    await t.mutation(internal.subscriptions.syncSubscription, {
      stripeCustomerId: "cus_update123",
      stripeSubscriptionId: "sub_update123",
      plan: "pro",
      status: "active",
      currentPeriodStart: new Date().toISOString(),
      currentPeriodEnd: new Date(
        Date.now() + 30 * 24 * 60 * 60 * 1000,
      ).toISOString(),
      stripePriceId: "price_pro_monthly",
      cancelAtPeriodEnd: false,
    });

    // Update it (e.g., plan change or cancel)
    await t.mutation(internal.subscriptions.syncSubscription, {
      stripeCustomerId: "cus_update123",
      stripeSubscriptionId: "sub_update123",
      plan: "pro",
      status: "canceled",
      currentPeriodStart: new Date().toISOString(),
      currentPeriodEnd: new Date(
        Date.now() + 30 * 24 * 60 * 60 * 1000,
      ).toISOString(),
      stripePriceId: "price_pro_monthly",
      cancelAtPeriodEnd: true,
    });

    const subs = await t.run(async (ctx) => {
      return ctx.db
        .query("subscriptions")
        .withIndex("by_stripeSubscriptionId", (q) =>
          q.eq("stripeSubscriptionId", "sub_update123"),
        )
        .collect();
    });

    // Should still be just one record (upsert)
    expect(subs).toHaveLength(1);
    expect(subs[0].status).toBe("canceled");
    expect(subs[0].cancelAtPeriodEnd).toBe(true);
  });
});

describe("recordBillingEvent mutation", () => {
  test("records events idempotently", async () => {
    const t = convexTest(schema, modules);
    const payload = { type: "invoice.paid", data: { amount: 999 } };

    // Record event first time
    await t.mutation(internal.subscriptions.recordBillingEvent, {
      stripeEventId: "evt_idempotent_1",
      eventType: "invoice.paid",
      payload,
    });

    // Record same event again (should be ignored)
    await t.mutation(internal.subscriptions.recordBillingEvent, {
      stripeEventId: "evt_idempotent_1",
      eventType: "invoice.paid",
      payload,
    });

    const events = await t.run(async (ctx) => {
      return ctx.db
        .query("billingEvents")
        .withIndex("by_stripeEventId", (q) =>
          q.eq("stripeEventId", "evt_idempotent_1"),
        )
        .collect();
    });

    // Should only have one record despite two calls
    expect(events).toHaveLength(1);
  });

  test("records event with userId", async () => {
    const t = convexTest(schema, modules);
    const userId = await createTestUser(t);

    await t.mutation(internal.subscriptions.recordBillingEvent, {
      stripeEventId: "evt_with_user",
      eventType: "customer.subscription.created",
      payload: { customer: "cus_test" },
      userId,
    });

    const events = await t.run(async (ctx) => {
      return ctx.db
        .query("billingEvents")
        .withIndex("by_userId", (q) => q.eq("userId", userId))
        .collect();
    });

    expect(events).toHaveLength(1);
    expect(events[0].stripeEventId).toBe("evt_with_user");
  });
});

describe("getSubscription query", () => {
  test("returns null for free users (no subscription)", async () => {
    const t = convexTest(schema, modules);
    const authSubject = "test-issuer|user_free";

    // Create user but no subscription
    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("users", {
        legacyKey: "test:user:free1",
        authSubject,
        email: "free@example.com",
        displayName: "Free User",
        createdAt: now,
        updatedAt: now,
      });
    });

    const asUser = t.withIdentity({ tokenIdentifier: authSubject });
    const result = await asUser.query(api.subscriptions.getSubscription, {});

    expect(result).toBeNull();
  });

  test("returns subscription for paid users", async () => {
    const t = convexTest(schema, modules);
    const authSubject = "test-issuer|user_paid1";

    // Create user with matching authSubject
    const userId = await t.run(async (ctx) => {
      const now = new Date().toISOString();
      return ctx.db.insert("users", {
        legacyKey: "test:user:paid1",
        authSubject,
        email: "paid@example.com",
        displayName: "Paid User",
        createdAt: now,
        updatedAt: now,
      });
    });

    // Create subscription for this user
    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:paid1",
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_paid1",
        stripeSubscriptionId: "sub_paid1",
        stripePriceId: "price_pro",
        plan: "pro",
        status: "active",
        currentPeriodStart: now,
        currentPeriodEnd: new Date(
          Date.now() + 30 * 24 * 60 * 60 * 1000,
        ).toISOString(),
      });
    });

    const asUser = t.withIdentity({ tokenIdentifier: authSubject });
    const result = await asUser.query(api.subscriptions.getSubscription, {});

    expect(result).not.toBeNull();
    expect(result!.plan).toBe("pro");
    expect(result!.status).toBe("active");
  });
});

describe("getFeatureAccess query", () => {
  test("returns correct access for free user", async () => {
    const t = convexTest(schema, modules);
    const authSubject = "test-issuer|user_freeaccess";

    // Create user
    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("users", {
        legacyKey: "test:user:freeaccess",
        authSubject,
        email: "free@example.com",
        displayName: "Free User",
        createdAt: now,
        updatedAt: now,
      });
    });

    const asUser = t.withIdentity({ tokenIdentifier: authSubject });
    const result = await asUser.query(api.subscriptions.getFeatureAccess, {
      feature: "autoEnforce",
    });

    expect(result.allowed).toBe(false);
    expect(result.reason).toBeDefined();
  });

  test("returns correct access for pro user", async () => {
    const t = convexTest(schema, modules);
    const authSubject = "test-issuer|user_proaccess";

    const userId = await t.run(async (ctx) => {
      const now = new Date().toISOString();
      return ctx.db.insert("users", {
        legacyKey: "test:user:proaccess",
        authSubject,
        email: "pro@example.com",
        displayName: "Pro User",
        createdAt: now,
        updatedAt: now,
      });
    });

    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:proaccess",
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_proaccess",
        stripeSubscriptionId: "sub_proaccess",
        stripePriceId: "price_pro",
        plan: "pro",
        status: "active",
        currentPeriodStart: now,
        currentPeriodEnd: new Date(
          Date.now() + 30 * 24 * 60 * 60 * 1000,
        ).toISOString(),
      });
    });

    const asUser = t.withIdentity({ tokenIdentifier: authSubject });
    const result = await asUser.query(api.subscriptions.getFeatureAccess, {
      feature: "autoEnforce",
    });

    expect(result.allowed).toBe(true);
  });

  test("returns correct access for team user", async () => {
    const t = convexTest(schema, modules);
    const authSubject = "test-issuer|user_teamaccess";

    const userId = await t.run(async (ctx) => {
      const now = new Date().toISOString();
      return ctx.db.insert("users", {
        legacyKey: "test:user:teamaccess",
        authSubject,
        email: "team@example.com",
        displayName: "Team User",
        createdAt: now,
        updatedAt: now,
      });
    });

    await t.run(async (ctx) => {
      const now = new Date().toISOString();
      await ctx.db.insert("subscriptions", {
        legacyKey: "test:sub:teamaccess",
        createdAt: now,
        updatedAt: now,
        userId,
        stripeCustomerId: "cus_teamaccess",
        stripeSubscriptionId: "sub_teamaccess",
        stripePriceId: "price_team",
        plan: "team",
        status: "active",
        currentPeriodStart: now,
        currentPeriodEnd: new Date(
          Date.now() + 30 * 24 * 60 * 60 * 1000,
        ).toISOString(),
        seats: 5,
      });
    });

    const asUser = t.withIdentity({ tokenIdentifier: authSubject });
    const result = await asUser.query(api.subscriptions.getFeatureAccess, {
      feature: "export",
    });

    expect(result.allowed).toBe(true);
  });
});
