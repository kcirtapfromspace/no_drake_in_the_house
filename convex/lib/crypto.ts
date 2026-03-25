/**
 * Token encryption/decryption using AES-256-GCM via the Web Crypto API.
 *
 * Only usable inside Convex **actions** (not mutations or queries) because
 * `crypto.subtle` operations are async and require the V8 action runtime.
 *
 * The encryption key is expected to be a base64-encoded 32-byte (256-bit)
 * value stored in the `OAUTH_ENCRYPTION_KEY` environment variable.
 *
 * Ciphertext format (all base64-encoded together):
 *   [ 12-byte IV | ciphertext+tag ]
 */

const IV_LENGTH = 12; // 96-bit IV recommended for AES-GCM
const ENCRYPTED_MIN_LENGTH = 32; // base64 of at least IV + some ciphertext

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Decode a base64 string into a Uint8Array. */
function base64ToBytes(b64: string): Uint8Array {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/** Encode a Uint8Array as a base64 string. */
function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

/** Import a raw 256-bit key for AES-GCM. */
async function importKey(keyBase64: string): Promise<CryptoKey> {
  const rawKey = base64ToBytes(keyBase64);
  if (rawKey.length !== 32) {
    throw new Error(
      `OAUTH_ENCRYPTION_KEY must decode to exactly 32 bytes (got ${rawKey.length}).`,
    );
  }
  return crypto.subtle.importKey(
    "raw",
    rawKey.buffer as ArrayBuffer,
    { name: "AES-GCM" },
    false,
    ["encrypt", "decrypt"],
  );
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Encrypt a plaintext token using AES-256-GCM.
 *
 * @param plaintext - The raw token value to encrypt.
 * @param keyBase64 - Base64-encoded 32-byte encryption key.
 * @returns A base64-encoded string containing `[IV || ciphertext+tag]`.
 */
export async function encryptToken(
  plaintext: string,
  keyBase64: string,
): Promise<string> {
  const key = await importKey(keyBase64);
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));
  const encoded = new TextEncoder().encode(plaintext);

  const cipherBuf = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    encoded,
  );

  // Combine IV + ciphertext into a single buffer
  const combined = new Uint8Array(IV_LENGTH + cipherBuf.byteLength);
  combined.set(iv, 0);
  combined.set(new Uint8Array(cipherBuf), IV_LENGTH);

  return bytesToBase64(combined);
}

/**
 * Decrypt a token that was encrypted with {@link encryptToken}.
 *
 * @param ciphertext - Base64-encoded `[IV || ciphertext+tag]`.
 * @param keyBase64  - Base64-encoded 32-byte encryption key.
 * @returns The original plaintext token.
 */
export async function decryptToken(
  ciphertext: string,
  keyBase64: string,
): Promise<string> {
  const key = await importKey(keyBase64);
  const combined = base64ToBytes(ciphertext);

  if (combined.length <= IV_LENGTH) {
    throw new Error("Ciphertext too short — cannot extract IV.");
  }

  const iv = combined.slice(0, IV_LENGTH);
  const data = combined.slice(IV_LENGTH);

  const plainBuf = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    key,
    data,
  );

  return new TextDecoder().decode(plainBuf);
}

/**
 * Heuristic check to determine whether a value looks like an encrypted token
 * (i.e. the base64-encoded output of {@link encryptToken}) as opposed to a
 * legacy plaintext OAuth token.
 *
 * Plaintext OAuth tokens are typically short alphanumeric/dash/underscore
 * strings or JWTs (three base64url segments separated by dots). Our encrypted
 * format produces a single base64 blob with no dots and a minimum length that
 * exceeds a bare IV.
 *
 * This is intentionally conservative: when in doubt it returns `false` so the
 * caller treats the value as plaintext, avoiding a decryption failure on
 * unencrypted legacy data.
 */
export function isEncrypted(value: string): boolean {
  if (!value || value.length < ENCRYPTED_MIN_LENGTH) {
    return false;
  }

  // Plaintext JWTs and many OAuth tokens contain dots — our cipher output never does.
  if (value.includes(".")) {
    return false;
  }

  // Must be valid standard base64 (A-Z, a-z, 0-9, +, /, =)
  // OAuth tokens often use dashes and underscores (base64url) which are NOT
  // in our output.
  if (/[_-]/.test(value)) {
    return false;
  }

  // Verify it decodes cleanly as base64 and is long enough to contain IV + at
  // least a minimal AES-GCM ciphertext (IV=12 + tag=16 + >=1 byte = 29).
  try {
    const decoded = atob(value);
    return decoded.length >= IV_LENGTH + 16 + 1;
  } catch {
    return false;
  }
}

/**
 * Read the encryption key from the Convex environment.
 * Throws a descriptive error when the variable is not set.
 */
export function getEncryptionKey(): string {
  const key = process.env.OAUTH_ENCRYPTION_KEY;
  if (!key) {
    throw new Error(
      "OAUTH_ENCRYPTION_KEY environment variable is not set. " +
        "Generate a 32-byte key with: openssl rand -base64 32",
    );
  }
  return key;
}
