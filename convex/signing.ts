"use node";

import { createSign } from "node:crypto";
import { v } from "convex/values";
import { internalAction } from "./_generated/server";

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
