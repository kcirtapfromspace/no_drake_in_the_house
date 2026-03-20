import type { AuthConfig } from "convex/server";

const domain =
  process.env.CONVEX_AUTH0_DOMAIN ?? "https://example.us.auth0.com";

const applicationIDs = [
  process.env.CONVEX_AUTH0_WEB_CLIENT_ID,
  process.env.CONVEX_AUTH0_ANDROID_CLIENT_ID,
].filter((value): value is string => Boolean(value && value.trim()));

export default {
  providers: (applicationIDs.length > 0
    ? applicationIDs
    : ["placeholder-auth0-client-id"]
  ).map((applicationID) => ({
    domain,
    applicationID,
  })),
} satisfies AuthConfig;
