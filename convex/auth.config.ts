import type { AuthConfig } from "convex/server";

export default {
  providers: [
    {
      domain: "https://api.nodrakeinthe.house",
      applicationID: "convex",
    },
  ],
} satisfies AuthConfig;
