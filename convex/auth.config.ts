import type { AuthConfig } from "convex/server";

export default {
  providers: [
    {
      domain:
        process.env.AUTH_ISSUER_URL ?? "https://api.nodrakeinthe.house",
      applicationID: "convex",
    },
  ],
} satisfies AuthConfig;
