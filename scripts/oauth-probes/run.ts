import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildRecord,
  filterDefinitions,
  REQUIRED_RESULT_FIELDS,
  type ProbeProvider,
  type ProbeRecord,
} from "../../convex/lib/oauthSyntheticProbes";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");

const DEFAULT_OUTPUT_PATH = path.resolve(
  repoRoot,
  "docs/evidence/oauth-synthetic-probes/latest.json",
);
const DEFAULT_STATE_PATH = path.resolve(
  repoRoot,
  "data/oauth-synthetic-probe-state.json",
);

type Args = {
  provider: ProbeProvider | "all";
  output: string;
  stateFile: string;
};

function nowIso(): string {
  return new Date().toISOString();
}

function usage(): string {
  return [
    "Usage: npm run probe:oauth -- [options]",
    "",
    "The CLI runner is always deterministic — it never calls live providers.",
    "",
    "Options:",
    "  --provider <spotify|apple|tidal|all>   Target provider (default: all)",
    "  --output <path>                        JSON artifact path",
    "  --state-file <path>                    State file for last_success tracking",
    "  --help                                 Show this help",
    "",
    "Examples:",
    "  npm run probe:oauth",
    "  npm run probe:oauth -- --provider spotify",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  const args: Args = {
    provider: "all",
    output: DEFAULT_OUTPUT_PATH,
    stateFile: DEFAULT_STATE_PATH,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];

    if (token === "--help" || token === "-h") {
      process.stdout.write(`${usage()}\n`);
      process.exit(0);
    }

    if (token === "--provider") {
      const value = argv[i + 1];
      if (!value) {
        throw new Error("--provider requires a value");
      }
      if (!["spotify", "apple", "tidal", "all"].includes(value)) {
        throw new Error(`Unsupported provider '${value}'`);
      }
      args.provider = value as ProbeProvider | "all";
      i += 1;
      continue;
    }

    if (token === "--output") {
      const value = argv[i + 1];
      if (!value) {
        throw new Error("--output requires a value");
      }
      args.output = path.resolve(process.cwd(), value);
      i += 1;
      continue;
    }

    if (token === "--state-file") {
      const value = argv[i + 1];
      if (!value) {
        throw new Error("--state-file requires a value");
      }
      args.stateFile = path.resolve(process.cwd(), value);
      i += 1;
      continue;
    }

    throw new Error(`Unknown argument '${token}'`);
  }

  return args;
}

type StateFile = { lastSuccessByProbeId: Record<string, string> };

async function loadState(filePath: string): Promise<StateFile> {
  try {
    const raw = await readFile(filePath, "utf8");
    const parsed = JSON.parse(raw) as Partial<StateFile>;
    return { lastSuccessByProbeId: parsed.lastSuccessByProbeId ?? {} };
  } catch {
    return { lastSuccessByProbeId: {} };
  }
}

async function saveState(filePath: string, state: StateFile): Promise<void> {
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, `${JSON.stringify(state, null, 2)}\n`, "utf8");
}

function toRepoRelative(absolutePath: string): string {
  const rel = path.relative(repoRoot, absolutePath);
  // If the artifact lives outside the repo (custom --output), keep the
  // basename only so we never leak machine-local absolute paths into
  // committed evidence.
  if (rel.startsWith("..") || path.isAbsolute(rel)) {
    return path.basename(absolutePath);
  }
  return rel;
}

function toEvidenceMarkdown(
  args: Args,
  results: ProbeRecord[],
  generatedAt: string,
): string {
  const lines: string[] = [];
  lines.push("# OAuth Synthetic Probe Evidence");
  lines.push("");
  lines.push(`- generated_at: ${generatedAt}`);
  lines.push(`- provider_target: ${args.provider}`);
  lines.push(`- records: ${results.length}`);
  lines.push("");

  for (const result of results) {
    lines.push(
      `## ${result.provider} | ${result.class} | ${result.status.toUpperCase()}`,
    );
    lines.push("");
    lines.push("```json");
    lines.push(JSON.stringify(result, null, 2));
    lines.push("```");
    lines.push("");
  }

  return `${lines.join("\n")}\n`;
}

function logHumanSummary(results: ProbeRecord[]): void {
  process.stderr.write("OAuth Synthetic Probe Results\n");
  process.stderr.write("================================\n");
  for (const result of results) {
    process.stderr.write(
      `[${result.status.toUpperCase()}] provider=${result.provider} flow=${result.flow} class=${result.class} simulation=${result.simulation_label}\n`,
    );
  }
}

function assertContract(results: ProbeRecord[]): void {
  for (const result of results) {
    for (const field of REQUIRED_RESULT_FIELDS) {
      if (!(field in result)) {
        throw new Error(
          `Probe contract violation: result is missing required field '${field}'`,
        );
      }
    }
  }
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const definitions = filterDefinitions(args.provider);

  if (definitions.length === 0) {
    throw new Error("No probe definitions matched the selected provider");
  }

  const state = await loadState(args.stateFile);
  const generatedAt = nowIso();

  const results: ProbeRecord[] = definitions.map((definition) => {
    const previousLastSuccess =
      state.lastSuccessByProbeId[definition.id] ?? null;
    const record = buildRecord(definition, nowIso(), previousLastSuccess);
    if (record.status === "pass" && record.last_success) {
      state.lastSuccessByProbeId[definition.id] = record.last_success;
    }
    return record;
  });

  assertContract(results);

  await saveState(args.stateFile, state);

  const outputJson = {
    generated_at: generatedAt,
    provider_target: args.provider,
    output_path: toRepoRelative(args.output),
    state_path: toRepoRelative(args.stateFile),
    probe_count: results.length,
    results,
  };

  const ndjsonPath = args.output.endsWith(".json")
    ? args.output.replace(/\.json$/, ".ndjson")
    : `${args.output}.ndjson`;

  const markdownPath = args.output.endsWith(".json")
    ? args.output.replace(/\.json$/, ".md")
    : `${args.output}.md`;

  await mkdir(path.dirname(args.output), { recursive: true });
  await writeFile(args.output, `${JSON.stringify(outputJson, null, 2)}\n`, "utf8");
  await writeFile(
    ndjsonPath,
    `${results.map((result) => JSON.stringify(result)).join("\n")}\n`,
    "utf8",
  );
  await writeFile(markdownPath, toEvidenceMarkdown(args, results, generatedAt), "utf8");

  logHumanSummary(results);

  process.stdout.write(`${JSON.stringify(outputJson, null, 2)}\n`);

  const failed = results.some((result) => result.status === "fail");
  if (failed) {
    process.exitCode = 1;
  }
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`OAuth probe runner failed: ${message}\n`);
  process.exitCode = 1;
});
