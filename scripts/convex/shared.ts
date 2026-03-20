import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { ConvexHttpClient } from "convex/browser";
import dotenv from "dotenv";
import { Client } from "pg";

dotenv.config({ path: ".env" });
dotenv.config({ path: ".env.local", override: true });
dotenv.config({ path: ".env.convex", override: true });
dotenv.config({ path: ".env.convex.local", override: true });

export function requireEnv(name: string) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable ${name}.`);
  }
  return value;
}

export function createPostgresClient() {
  return new Client({
    connectionString: requireEnv("DATABASE_URL"),
  });
}

export function createConvexClient() {
  return new ConvexHttpClient(requireEnv("CONVEX_URL"));
}

export function toIsoString(value: unknown, fallback = new Date().toISOString()) {
  if (!value) {
    return fallback;
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number") {
    return new Date(value).toISOString();
  }
  return fallback;
}

export function sanitizeJson(value: unknown): any {
  if (value === null || value === undefined) {
    return undefined;
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Buffer.isBuffer(value)) {
    return value.toString("base64");
  }
  if (Array.isArray(value)) {
    return value.map((entry) => sanitizeJson(entry));
  }
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
        key,
        sanitizeJson(entry),
      ]),
    );
  }
  return value;
}

export function chunk<T>(values: T[], size: number) {
  const batches: T[][] = [];
  for (let index = 0; index < values.length; index += size) {
    batches.push(values.slice(index, index + size));
  }
  return batches;
}

export function buildLifecycle(row: Record<string, unknown>) {
  const createdAt = toIsoString(
    row.created_at ??
      row.added_at ??
      row.started_at ??
      row.published_at ??
      row.posted_at ??
      row.scan_started_at,
  );

  const updatedAt = toIsoString(
    row.updated_at ??
      row.last_updated ??
      row.last_synced ??
      row.completed_at ??
      row.scan_completed_at ??
      row.last_polled_at ??
      row.verified_at ??
      createdAt,
    createdAt,
  );

  return { createdAt, updatedAt };
}

export function ensureLegacyKey(
  row: Record<string, unknown>,
  fallbackParts: unknown[],
) {
  if (typeof row.id === "string") {
    return row.id;
  }

  return fallbackParts
    .filter((part) => part !== null && part !== undefined && part !== "")
    .map((part) => String(part))
    .join(":");
}

export async function tableExists(client: Client, tableName: string) {
  const result = await client.query<{ exists: boolean }>(
    `
      select exists (
        select 1
        from information_schema.tables
        where table_schema = 'public'
          and table_name = $1
      ) as exists
    `,
    [tableName],
  );
  return result.rows[0]?.exists ?? false;
}

export async function writeJsonFile(relativePath: string, payload: unknown) {
  const absolutePath = path.resolve(relativePath);
  await mkdir(path.dirname(absolutePath), { recursive: true });
  await writeFile(absolutePath, JSON.stringify(payload, null, 2));
  return absolutePath;
}

export class MappingStore {
  private values = new Map<string, string>();

  remember(table: string, legacyKey: string, convexId: string) {
    this.values.set(`${table}:${legacyKey}`, convexId);
  }

  rememberBatch(
    table: string,
    mappings: Array<{ legacyKey: string; convexId: string }>,
  ) {
    for (const mapping of mappings) {
      this.remember(table, mapping.legacyKey, mapping.convexId);
    }
  }

  get(table: string, legacyKey: unknown) {
    if (typeof legacyKey !== "string" || legacyKey.length === 0) {
      return undefined;
    }
    return this.values.get(`${table}:${legacyKey}`);
  }

  require(table: string, legacyKey: unknown) {
    const value = this.get(table, legacyKey);
    if (!value) {
      throw new Error(
        `Missing Convex mapping for ${table}:${String(legacyKey)}. Import dependencies first.`,
      );
    }
    return value;
  }
}
