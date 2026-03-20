/* eslint-disable */
/**
 * Generated `api` utility.
 *
 * This checked-in variant provides a stable typed surface before the repo is
 * connected to a specific Convex deployment.
 */
import type { ApiFromModules, FilterApi, FunctionReference } from "convex/server";
import { anyApi } from "convex/server";
import type * as analytics from "../analytics.js";
import type * as categories from "../categories.js";
import type * as crons from "../crons.js";
import type * as dnp from "../dnp.js";
import type * as extension from "../extension.js";
import type * as graph from "../graph.js";
import type * as http from "../http.js";
import type * as migration from "../migration.js";
import type * as offenses from "../offenses.js";
import type * as providerConnections from "../providerConnections.js";
import type * as users from "../users.js";

const fullApi: ApiFromModules<{
  analytics: typeof analytics;
  categories: typeof categories;
  crons: typeof crons;
  dnp: typeof dnp;
  extension: typeof extension;
  graph: typeof graph;
  http: typeof http;
  migration: typeof migration;
  offenses: typeof offenses;
  providerConnections: typeof providerConnections;
  users: typeof users;
}> = anyApi as any;

export const api: FilterApi<typeof fullApi, FunctionReference<any, "public">> =
  anyApi as any;
export const internal: FilterApi<
  typeof fullApi,
  FunctionReference<any, "internal">
> = anyApi as any;
