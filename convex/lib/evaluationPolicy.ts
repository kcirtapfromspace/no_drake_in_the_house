/** Versioned domain rules shared by intake, review and projections. */
export const POLICY_VERSION = "evidence-v1";
export const DEFAULT_JEV_MODEL = "jev-1.13.0";
export const MAX_SOURCE_LENGTH = 24_000;
export const MAX_ATTEMPTS = 4;
export const LEASE_MS = 180_000;
export const CATEGORIES = {
  sexual_misconduct: "Sexual assault, abuse or harassment",
  domestic_violence: "Violence or abuse against an intimate partner or household member",
  hate_speech: "Targeted hateful statements about a protected group",
  racism: "Racial discrimination or racist conduct",
  antisemitism: "Antisemitic discrimination or hateful conduct",
  homophobia: "Anti-LGBTQ discrimination or hateful conduct",
  child_abuse: "Abuse or exploitation of a child; not musical minor keys",
  animal_cruelty: "Abuse, neglect or deliberate cruelty to animals",
  financial_crimes: "Fraud, embezzlement, money laundering or similar financial offenses",
  drug_offenses: "Illegal drug possession, distribution or trafficking",
  violent_crimes: "Physical violent offenses outside a domestic context",
  harassment: "Stalking, threats or targeted harassment",
  plagiarism: "Uncredited copying or copyright infringement",
  certified_creeper: "Reported grooming or exploitative relationships involving minors",
} as const;
export type Category = keyof typeof CATEGORIES;
export const SEVERITIES = ["minor", "moderate", "severe", "egregious"] as const;
export type Severity = typeof SEVERITIES[number];
export const PROCEDURAL_STATES = ["alleged", "charged", "convicted", "acquitted", "dismissed", "settled", "not_stated"] as const;

export function normalizedSeverity(value: string): Severity {
  const aliases: Record<string, Severity> = {
    low: "minor", medium: "moderate", high: "severe", critical: "egregious",
    minor: "minor", moderate: "moderate", severe: "severe", egregious: "egregious",
  };
  const severity = aliases[value];
  if (!severity) throw new Error("Unknown severity");
  return severity;
}

export function isApprovedOffense(offense: { status?: string }): boolean {
  return offense.status === "verified";
}

export function isReviewer(user: { roles?: string[] }): boolean {
  return (user.roles ?? []).some((role) => role === "owner" || role === "reviewer");
}

export function canonicalSourceUrl(raw: string): string {
  if (raw.length > 2048) throw new Error("Source URL is too long");
  const url = new URL(raw);
  // Sources are public web pages; never send credentials or local targets to a fetcher.
  const host = url.hostname.toLowerCase();
  if (!(["https:", "http:"].includes(url.protocol)) || url.username || url.password ||
      !host.includes(".") || host.endsWith(".localhost") || host.endsWith(".local") ||
      /^[\d.]+$/.test(host) || host.includes(":")) {
    throw new Error("A public HTTP(S) source URL is required");
  }
  url.hash = "";
  return url.toString();
}
