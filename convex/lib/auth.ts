import { ConvexError } from "convex/values";
import type { MutationCtx, QueryCtx } from "../_generated/server";

type AnyCtx = MutationCtx | QueryCtx;

export function nowIso() {
  return new Date().toISOString();
}

export async function requireIdentity(ctx: AnyCtx) {
  const identity = await ctx.auth.getUserIdentity();
  if (!identity) {
    throw new ConvexError("Authentication required.");
  }
  return identity;
}

export async function getCurrentUser(ctx: AnyCtx) {
  const identity = await requireIdentity(ctx);
  const user = await ctx.db
    .query("users")
    .withIndex("by_authSubject", (q) =>
      q.eq("authSubject", identity.tokenIdentifier),
    )
    .unique();

  return { identity, user };
}

export async function requireCurrentUser(ctx: AnyCtx) {
  const { identity, user } = await getCurrentUser(ctx);
  if (!user) {
    throw new ConvexError(
      `Authenticated identity ${identity.tokenIdentifier} has not been synced.`,
    );
  }
  return { identity, user };
}

export async function requireOwner(ctx: AnyCtx) {
  const { identity, user } = await requireCurrentUser(ctx);
  const roles = user.roles ?? [];
  if (!roles.includes("owner")) {
    throw new ConvexError("Owner access required.");
  }
  return { identity, user };
}

function resolveDisplayName(identity: Awaited<ReturnType<typeof requireIdentity>>) {
  return (
    identity.name ??
    identity.nickname ??
    identity.preferredUsername ??
    identity.email ??
    "No Drake in the House user"
  );
}

export async function upsertCurrentUserFromIdentity(ctx: MutationCtx) {
  const identity = await requireIdentity(ctx);
  const now = nowIso();
  const authSubject = identity.tokenIdentifier;

  const existingBySubject = await ctx.db
    .query("users")
    .withIndex("by_authSubject", (q) => q.eq("authSubject", authSubject))
    .unique();

  const email = identity.email?.toLowerCase();
  const emailVerified = identity.emailVerified ?? false;

  if (existingBySubject) {
    await ctx.db.patch(existingBySubject._id, {
      email,
      emailVerified,
      displayName: resolveDisplayName(identity),
      avatarUrl: identity.pictureUrl,
      lastLoginAt: now,
      updatedAt: now,
    });
    return await ctx.db.get(existingBySubject._id);
  }

  let matchedUser = null;
  if (email) {
    matchedUser = await ctx.db
      .query("users")
      .withIndex("by_email", (q) => q.eq("email", email))
      .unique();
  }

  if (matchedUser) {
    await ctx.db.patch(matchedUser._id, {
      authSubject,
      email,
      emailVerified,
      displayName: resolveDisplayName(identity),
      avatarUrl: identity.pictureUrl,
      lastLoginAt: now,
      updatedAt: now,
    });
    return await ctx.db.get(matchedUser._id);
  }

  const insertedId = await ctx.db.insert("users", {
    legacyKey: `runtime:user:${authSubject}`,
    legacyUserId: undefined,
    authSubject,
    email,
    emailVerified,
    displayName: resolveDisplayName(identity),
    avatarUrl: identity.pictureUrl,
    settings: {},
    roles: [],
    totpEnabled: false,
    lastLoginAt: now,
    metadata: {
      issuer: identity.issuer,
      subject: identity.subject,
    },
    createdAt: now,
    updatedAt: now,
  });

  return await ctx.db.get(insertedId);
}
