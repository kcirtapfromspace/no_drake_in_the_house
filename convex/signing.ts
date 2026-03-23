"use node";

import { createSign } from "node:crypto";
import { v } from "convex/values";
import { internalAction } from "./_generated/server";

export const getDeveloperToken = internalAction({
  args: {},
  handler: async () => {
    const teamId = process.env.APPLE_MUSIC_TEAM_ID;
    const keyId = process.env.APPLE_MUSIC_KEY_ID;
    const privateKey = process.env.APPLE_MUSIC_PRIVATE_KEY;

    if (!teamId || !keyId || !privateKey) {
      return {
        developer_token: null,
        error: "Apple Music credentials not configured.",
      };
    }

    return {
      developer_token: `apple_dev_${teamId}_${keyId}`,
      expires_in: 3600,
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
