import type { AuthConfig } from "convex/server";

export default {
  providers: [
    {
      domain: process.env.AUTH_ISSUER_URL!,
      applicationID: "convex",
    },
  ],
} satisfies AuthConfig;
