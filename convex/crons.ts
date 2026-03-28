import { cronJobs } from "convex/server";
import { api, internal } from "./_generated/api";

const crons = cronJobs();

crons.interval(
  "refresh-global-extension-snapshot",
  { hours: 1 },
  api.extension.refreshGlobalSnapshot,
  {},
);

crons.daily(
  "snapshot-trouble-scores",
  { hourUTC: 4, minuteUTC: 0 },
  internal.analytics.snapshotTroubleScores,
);

crons.interval(
  "refresh-expiring-oauth-tokens",
  { minutes: 30 },
  (internal as any).providerOAuth.refreshExpiringTokens,
  {},
);

crons.interval(
  "promote-classifications-to-offenses",
  { hours: 6 },
  (internal as any).offensePipeline.promoteClassifications,
  {},
);

crons.interval(
  "rebuild-offending-artist-index",
  { hours: 6 },
  (internal as any).offensePipeline.rebuildOffendingArtistIndex,
  {},
);

crons.daily(
  "sweep-stale-offense-summaries",
  { hourUTC: 5, minuteUTC: 0 },
  (internal as any).offensePipeline.dailySweep,
);

// Daily investigation of library artists for new evidence.
// Uses string reference because evidenceFinder is new and not yet in generated types.
// After running `npx convex dev` once, replace with: internal.evidenceFinder.dailyInvestigation
crons.daily(
  "investigate-library-artists",
  { hourUTC: 3, minuteUTC: 0 },
  "evidenceFinder:dailyInvestigation" as any,
);

export default crons;
