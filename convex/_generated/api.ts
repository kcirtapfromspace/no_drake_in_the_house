/* eslint-disable */
/**
 * Generated `api` utility.
 *
 * THIS CODE IS AUTOMATICALLY GENERATED.
 *
 * To regenerate, run `npx convex dev`.
 * @module
 */
import type { ApiFromModules, FilterApi, FunctionReference } from "convex/server";
import { anyApi } from "convex/server";
import type * as analytics from "../analytics.js";
import type * as appleMusic from "../appleMusic.js";
import type * as artistProfile from "../artistProfile.js";
import type * as billing from "../billing.js";
import type * as billing_helpers from "../billing_helpers.js";
import type * as categories from "../categories.js";
import type * as community from "../community.js";
import type * as crons from "../crons.js";
import type * as dnp from "../dnp.js";
import type * as enforcement from "../enforcement.js";
import type * as evidenceFinder from "../evidenceFinder.js";
import type * as extension from "../extension.js";
import type * as graph from "../graph.js";
import type * as http from "../http.js";
import type * as library from "../library.js";
import type * as librarySyncActions from "../librarySyncActions.js";
import type * as migration from "../migration.js";
import type * as news from "../news.js";
import type * as newsIngestion from "../newsIngestion.js";
import type * as offensePipeline from "../offensePipeline.js";
import type * as offenses from "../offenses.js";
import type * as providerConnections from "../providerConnections.js";
import type * as providerOAuth from "../providerOAuth.js";
import type * as sanitizer from "../sanitizer.js";
import type * as signing from "../signing.js";
import type * as stripeActions from "../stripeActions.js";
import type * as stripeHelpers from "../stripeHelpers.js";
import type * as subscriptions from "../subscriptions.js";
import type * as sync from "../sync.js";
import type * as users from "../users.js";

const fullApi: ApiFromModules<{
  analytics: typeof analytics;
  appleMusic: typeof appleMusic;
  artistProfile: typeof artistProfile;
  billing: typeof billing;
  billing_helpers: typeof billing_helpers;
  categories: typeof categories;
  community: typeof community;
  crons: typeof crons;
  dnp: typeof dnp;
  enforcement: typeof enforcement;
  evidenceFinder: typeof evidenceFinder;
  extension: typeof extension;
  graph: typeof graph;
  http: typeof http;
  library: typeof library;
  librarySyncActions: typeof librarySyncActions;
  migration: typeof migration;
  news: typeof news;
  newsIngestion: typeof newsIngestion;
  offensePipeline: typeof offensePipeline;
  offenses: typeof offenses;
  providerConnections: typeof providerConnections;
  providerOAuth: typeof providerOAuth;
  sanitizer: typeof sanitizer;
  signing: typeof signing;
  stripeActions: typeof stripeActions;
  stripeHelpers: typeof stripeHelpers;
  subscriptions: typeof subscriptions;
  sync: typeof sync;
  users: typeof users;
}> = anyApi as any;

export const api: FilterApi<typeof fullApi, FunctionReference<any, "public">> =
  anyApi as any;
export const internal: FilterApi<
  typeof fullApi,
  FunctionReference<any, "internal">
> = anyApi as any;
