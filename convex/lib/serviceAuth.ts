/**
 * Service-to-service authentication via shared secret.
 *
 * Uses a simple shared secret (`NDITH_SERVICE_KEY`) sent as an
 * `X-Service-Key` header. This avoids the JWT-based approach that
 * required a PostgreSQL user lookup on the Rust backend side.
 *
 * Requires `NDITH_SERVICE_KEY` env var (same value in Convex and Rust backend).
 */

/**
 * Build auth headers for backend requests.
 * Returns headers object with X-Service-Key if NDITH_SERVICE_KEY is available.
 */
export function serviceAuthHeaders(): Record<string, string> {
  const serviceKey = process.env.NDITH_SERVICE_KEY;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (serviceKey) {
    headers["X-Service-Key"] = serviceKey;
  }
  return headers;
}
