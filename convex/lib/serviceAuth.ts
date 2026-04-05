/**
 * JIT service-to-service JWT generation.
 *
 * Generates a short-lived (5 min) JWT signed with the shared JWT_SECRET
 * so Convex actions can authenticate against the Rust backend services.
 * No static tokens to manage — each request gets a fresh token.
 *
 * Requires `NDITH_JWT_SECRET` env var (same value as the backend's JWT_SECRET).
 */
import { SignJWT } from "jose";

const SERVICE_USER_ID = "00000000-0000-0000-0000-000000000001";

/**
 * Generate a short-lived service JWT for backend API calls.
 * Returns the Bearer token string, or null if JWT_SECRET is not configured.
 */
export async function generateServiceToken(): Promise<string | null> {
  const secret = process.env.NDITH_JWT_SECRET;
  if (!secret) return null;

  const key = new TextEncoder().encode(secret);
  const token = await new SignJWT({ sub: SERVICE_USER_ID, role: "service" })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime("5m")
    .sign(key);

  return token;
}

/**
 * Build auth headers for backend requests.
 * Returns headers object with Authorization if JWT_SECRET is available.
 */
export async function serviceAuthHeaders(): Promise<Record<string, string>> {
  const token = await generateServiceToken();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  return headers;
}
