"use node";

import { createSign } from "node:crypto";
import { v } from "convex/values";
import { internalAction } from "./_generated/server";

export const getDeveloperToken = internalAction({
  args: {},
  handler: async () => {
    const teamId =
      process.env.APPLE_MUSIC_TEAM_ID || process.env.APPLE_TEAM_ID;
    const keyId =
      process.env.APPLE_MUSIC_KEY_ID || process.env.APPLE_KEY_ID;
    const privateKey =
      process.env.APPLE_MUSIC_PRIVATE_KEY || process.env.APPLE_PRIVATE_KEY;
    // For MusicKit Web, the origin claim must be a web origin (e.g. https://example.com).
    // APPLE_MUSIC_ORIGIN is preferred; APPLE_MUSIC_BUNDLE_ID is only used if it
    // looks like a URL (not an iOS bundle ID like com.example.app).
    const rawOrigin =
      process.env.APPLE_MUSIC_ORIGIN ||
      process.env.APPLE_MUSIC_BUNDLE_ID ||
      process.env.APPLE_BUNDLE_ID;
    const webOrigin =
      rawOrigin && rawOrigin.startsWith("http") ? rawOrigin : undefined;

    if (!teamId || !keyId || !privateKey) {
      const missing = [
        !teamId && "APPLE_MUSIC_TEAM_ID or APPLE_TEAM_ID",
        !keyId && "APPLE_MUSIC_KEY_ID or APPLE_KEY_ID",
        !privateKey && "APPLE_MUSIC_PRIVATE_KEY or APPLE_PRIVATE_KEY",
      ].filter(Boolean);
      return {
        developer_token: null,
        error: `Apple Music credentials not configured. Missing: ${missing.join(", ")}`,
      };
    }

    // Build a proper Apple Music developer token (ES256 JWT)
    const now = Math.floor(Date.now() / 1000);
    const header = Buffer.from(
      JSON.stringify({ alg: "ES256", kid: keyId }),
    ).toString("base64url");
    const payload = Buffer.from(
      JSON.stringify({
        iss: teamId,
        iat: now,
        exp: now + 15777000, // 6 months
        ...(webOrigin ? { origin: [webOrigin] } : {}),
      }),
    ).toString("base64url");

    const signature = createSign("SHA256")
      .update(`${header}.${payload}`)
      .end()
      .sign(
        { key: privateKey.replace(/\\n/g, "\n"), dsaEncoding: "ieee-p1363" },
        "base64url",
      );

    return {
      developer_token: `${header}.${payload}.${signature}`,
      expires_in: 15777000,
    };
  },
});

export const signPayload = internalAction({
  args: {
    payload: v.string(),
    privateKeyEnvVar: v.optional(v.string()),
  },
  handler: async (_ctx, args) => {
    const envVar = args.privateKeyEnvVar ?? "EXTENSION_SIGNING_PRIVATE_KEY_PEM";
    const privateKey = process.env[envVar];
    if (!privateKey) {
      return "";
    }
    return createSign("RSA-SHA256")
      .update(args.payload)
      .end()
      .sign(privateKey, "base64");
  },
});
