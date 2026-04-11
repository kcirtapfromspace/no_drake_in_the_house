"use node";

import { ConvexError, v } from "convex/values";
import { action, internalAction } from "./_generated/server";
import type { ActionCtx } from "./_generated/server";
import { internal, api } from "./_generated/api";
import type { Id } from "./_generated/dataModel";

// ---------------------------------------------------------------------------
// Category keyword lists (ported from Rust offense_classifier.rs)
// ---------------------------------------------------------------------------

const CATEGORY_KEYWORDS: Record<string, string[]> = {
  sexual_misconduct: [
    "sexual assault", "sexual harassment", "rape", "groping",
    "inappropriate", "misconduct", "metoo", "#metoo",
    "sexual abuse", "molestation", "predator",
  ],
  domestic_violence: [
    "domestic violence", "domestic abuse", "beat", "hit",
    "assault", "battery", "restraining order", "abuse",
    "physical altercation", "attacked",
  ],
  hate_speech: [
    "hate speech", "slur", "offensive comments", "racist remarks",
    "discrimination", "bigot", "hateful", "derogatory",
  ],
  racism: [
    "racist", "racism", "racial slur", "n-word",
    "blackface", "white supremacy", "segregation", "racial discrimination",
  ],
  antisemitism: [
    "antisemit", "anti-semit", "holocaust", "nazi",
    "hitler", "concentration camp", "zionist conspiracy",
  ],
  homophobia: [
    "homophobic", "homophobia", "anti-gay", "anti-lgbtq",
    "transphobic", "transphobia",
  ],
  child_abuse: [
    "child abuse", "minor", "underage", "pedophile",
    "child exploitation", "grooming",
  ],
  animal_cruelty: [
    "animal cruelty", "animal abuse", "dogfighting",
    "dog fighting", "animal neglect", "animal torture",
  ],
  financial_crimes: [
    "fraud", "embezzlement", "money laundering", "tax evasion",
    "scam", "ponzi", "crypto scam", "nft scam",
  ],
  drug_offenses: [
    "drug trafficking", "drug possession", "cocaine", "heroin",
    "fentanyl", "drug arrest", "narcotics",
  ],
  violent_crimes: [
    "murder", "killed", "shooting", "stabbing",
    "assault", "manslaughter", "attempted murder", "gun", "weapon",
  ],
  harassment: [
    "harassment", "stalking", "cyberbullying", "threats",
    "intimidation", "bullying", "doxing", "death threats",
  ],
  plagiarism: [
    "plagiarism", "plagiarized", "plagiarised", "copied",
    "stolen song", "ghostwriter controversy", "uncredited",
    "copyright infringement", "sampling without permission", "music theft",
  ],
  certified_creeper: [
    "grooming", "underage girlfriend", "age gap",
    "dating a minor", "inappropriate relationship", "teenage girlfriend",
  ],
};

// ---------------------------------------------------------------------------
// Severity mapping (highest keyword match wins)
// ---------------------------------------------------------------------------

const SEVERITY_KEYWORDS: Record<string, Record<string, string>> = {
  sexual_misconduct: { rape: "critical", assault: "high", harassment: "medium" },
  domestic_violence: { hospitalized: "critical", beat: "high", "restraining order": "medium" },
  violent_crimes: { murder: "critical", shooting: "critical", assault: "high" },
  child_abuse: { pedophile: "critical", "child abuse": "critical", grooming: "critical" },
  animal_cruelty: { dogfighting: "critical", torture: "critical", cruelty: "high" },
  harassment: { "death threats": "critical", stalking: "high", harassment: "medium" },
  financial_crimes: { fraud: "high", embezzlement: "high", scam: "medium" },
  drug_offenses: { trafficking: "high", fentanyl: "high", possession: "low" },
  certified_creeper: { grooming: "critical", underage: "critical" },
};

// ---------------------------------------------------------------------------
// Domain credibility scores
// ---------------------------------------------------------------------------

const MAJOR_NEWS_DOMAINS = [
  "nytimes.com", "bbc.com", "bbc.co.uk", "cnn.com",
  "theguardian.com", "reuters.com", "apnews.com",
  "washingtonpost.com", "nbcnews.com", "abcnews.go.com",
];

const NEWS_MAGAZINE_DOMAINS = [
  "billboard.com", "rollingstone.com", "pitchfork.com",
  "variety.com", "tmz.com", "hollywoodreporter.com",
  "complex.com", "vice.com", "buzzfeednews.com",
];

function computeCredibility(url: string): number {
  try {
    const hostname = new URL(url).hostname.replace(/^www\./, "");
    if (MAJOR_NEWS_DOMAINS.some((d) => hostname.endsWith(d))) return 4.5;
    if (NEWS_MAGAZINE_DOMAINS.some((d) => hostname.endsWith(d))) return 4;
    if (hostname.endsWith("wikipedia.org")) return 3.5;
    return 2;
  } catch {
    return 2;
  }
}

// ---------------------------------------------------------------------------
// Keyword classifier
// ---------------------------------------------------------------------------

function classifyContent(
  text: string,
  hintCategory?: string,
): { category: string; severity: string; matchedKeywords: string[] } | null {
  const lower = text.toLowerCase();

  const categoriesToCheck = hintCategory
    ? [hintCategory, ...Object.keys(CATEGORY_KEYWORDS).filter((c) => c !== hintCategory)]
    : Object.keys(CATEGORY_KEYWORDS);

  for (const category of categoriesToCheck) {
    const keywords = CATEGORY_KEYWORDS[category];
    if (!keywords) continue;

    const matched = keywords.filter((kw) => lower.includes(kw));
    if (matched.length === 0) continue;

    let severity = "medium";
    const severityMap = SEVERITY_KEYWORDS[category];
    if (severityMap) {
      const SEVERITY_ORDER: Record<string, number> = {
        low: 0, medium: 1, high: 2, critical: 3,
      };
      for (const kw of matched) {
        for (const [sevKw, sev] of Object.entries(severityMap)) {
          if (kw.includes(sevKw) || lower.includes(sevKw)) {
            if ((SEVERITY_ORDER[sev] ?? 0) > (SEVERITY_ORDER[severity] ?? 0)) {
              severity = sev;
            }
          }
        }
      }
    }

    return { category, severity, matchedKeywords: matched };
  }

  return null;
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

type VerifyResult =
  | { verified: false; reason: string }
  | {
      verified: true;
      offenseId: Id<"artistOffenses">;
      category: string;
      credibilityScore: number;
    };

// ---------------------------------------------------------------------------
// Core verification logic (shared by both actions)
// ---------------------------------------------------------------------------

async function runVerification(
  ctx: ActionCtx,
  args: {
    artistId: Id<"artists">;
    url: string;
    category?: string;
    userId?: Id<"users">;
  },
): Promise<VerifyResult> {
  // a) Fetch artist name
  const artist: { name: string; canonicalName: string } | null =
    await ctx.runQuery(internal.evidenceFinder._getArtistName, {
      artistId: args.artistId,
    });

  if (!artist) {
    return { verified: false, reason: "Artist not found" };
  }

  // b) Call Firecrawl to scrape the URL
  const firecrawlKey = process.env.FIRECRAWL_API_KEY;
  if (!firecrawlKey) {
    throw new ConvexError("FIRECRAWL_API_KEY is not configured.");
  }

  const scrapeResponse = await fetch("https://api.firecrawl.dev/v2/scrape", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${firecrawlKey}`,
    },
    body: JSON.stringify({
      url: args.url,
      formats: ["markdown"],
      onlyMainContent: true,
    }),
  });

  if (!scrapeResponse.ok) {
    const errorText = await scrapeResponse.text().catch(() => "");
    return {
      verified: false,
      reason: `Failed to scrape URL (HTTP ${scrapeResponse.status}): ${errorText.slice(0, 200)}`,
    };
  }

  const scrapeResult = await scrapeResponse.json();
  const markdown: string = scrapeResult?.data?.markdown ?? "";

  if (!markdown || markdown.length < 50) {
    return { verified: false, reason: "Could not extract meaningful content from URL" };
  }

  // c) Check if content mentions the artist (case-insensitive)
  const artistLower = artist.canonicalName.toLowerCase();
  if (!markdown.toLowerCase().includes(artistLower)) {
    return { verified: false, reason: "Article does not mention this artist" };
  }

  // d) Run keyword classification
  const classification = classifyContent(markdown, args.category);
  if (!classification) {
    return { verified: false, reason: "No offense-related content found" };
  }

  // e) Compute credibility score
  const credibilityScore = computeCredibility(args.url);

  // f) Create offense and link evidence
  const offenseResult: { id: Id<"artistOffenses">; upserted: string } =
    await ctx.runMutation(api.newsIngestion.createOffenseFromResearch, {
      artistId: args.artistId,
      category: classification.category,
      severity: classification.severity,
      title: `${classification.category.replace(/_/g, " ")} — user-submitted evidence`,
      description: `Verified from: ${args.url}`,
      confidence: Math.min(classification.matchedKeywords.length * 0.2, 0.8),
      sourceArticleUrl: args.url,
    });

  const excerpt = markdown.slice(0, 500);

  await ctx.runMutation(api.newsIngestion.linkOffenseEvidence, {
    offenseId: offenseResult.id,
    sourceUrl: args.url,
    title: scrapeResult?.data?.metadata?.title ?? args.url,
    excerpt,
    credibilityScore,
  });

  return {
    verified: true,
    offenseId: offenseResult.id,
    category: classification.category,
    credibilityScore,
  };
}

// ---------------------------------------------------------------------------
// verifyAndIngestUrl — internalAction (callable from other actions/scheduler)
// ---------------------------------------------------------------------------

export const verifyAndIngestUrl = internalAction({
  args: {
    artistId: v.id("artists"),
    url: v.string(),
    category: v.optional(v.string()),
    userId: v.optional(v.id("users")),
  },
  handler: async (ctx, args): Promise<VerifyResult> => {
    return runVerification(ctx, {
      artistId: args.artistId,
      url: args.url,
      category: args.category ?? undefined,
      userId: args.userId ?? undefined,
    });
  },
});

// ---------------------------------------------------------------------------
// submitEvidence — public action (authenticated, rate-limited)
// ---------------------------------------------------------------------------

export const submitEvidence = action({
  args: {
    artistId: v.id("artists"),
    url: v.string(),
    category: v.optional(v.string()),
  },
  handler: async (ctx, args): Promise<VerifyResult> => {
    // Auth check
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) {
      throw new ConvexError("Authentication required.");
    }

    // Look up user
    const user: { _id: Id<"users"> } | null = await ctx.runQuery(
      internal.evidenceVerifierHelpers._getCurrentUserByToken,
      { tokenIdentifier: identity.tokenIdentifier },
    );
    if (!user) {
      throw new ConvexError("User account not found.");
    }

    // Rate limit: max 10 submissions per user per day
    const todayCount: number = await ctx.runQuery(
      internal.evidenceVerifierHelpers._countUserSubmissionsToday,
      { userId: user._id },
    );
    if (todayCount >= 10) {
      throw new ConvexError(
        "Rate limit exceeded: maximum 10 evidence submissions per day.",
      );
    }

    // Run verification directly (same runtime — no need for ctx.runAction)
    return runVerification(ctx, {
      artistId: args.artistId,
      url: args.url,
      category: args.category ?? undefined,
      userId: user._id,
    });
  },
});
