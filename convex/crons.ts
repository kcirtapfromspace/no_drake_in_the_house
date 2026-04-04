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

crons.daily(
  "snapshot-catalog-metrics",
  { hourUTC: 4, minuteUTC: 30 },
  internal.analytics.snapshotCatalogMetrics,
);

crons.interval(
  "refresh-expiring-oauth-tokens",
  { minutes: 30 },
  internal.providerOAuth.refreshExpiringTokens,
  {},
);

crons.interval(
  "promote-classifications-to-offenses",
  { hours: 6 },
  internal.offensePipeline.promoteClassifications,
  {},
);

crons.interval(
  "rebuild-offending-artist-index",
  { hours: 6 },
  internal.offensePipeline.rebuildOffendingArtistIndex,
  {},
);

crons.daily(
  "sweep-stale-offense-summaries",
  { hourUTC: 5, minuteUTC: 0 },
  internal.offensePipeline.dailySweep,
);

// Daily investigation of library artists for new evidence.
crons.daily(
  "investigate-library-artists",
  { hourUTC: 3, minuteUTC: 0 },
  internal.evidenceFinder.dailyInvestigation,
);

export default crons;
