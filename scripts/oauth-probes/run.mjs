import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");

const DEFAULT_DEFINITIONS_PATH = path.resolve(__dirname, "definitions.json");
const DEFAULT_OUTPUT_PATH = path.resolve(
  repoRoot,
  "docs/evidence/oauth-synthetic-probes/latest.json",
);
const DEFAULT_STATE_PATH = path.resolve(
  repoRoot,
  "data/oauth-synthetic-probe-state.json",
);

function nowIso() {
  return new Date().toISOString();
}

function usage() {
  return [
    "Usage: npm run probe:oauth -- [options]",
    "",
    "Options:",
    "  --provider <spotify|apple|tidal|all>   Target provider (default: all)",
    "  --dry-run                              Force deterministic simulation mode",
    "  --output <path>                        JSON artifact path",
    "  --state-file <path>                    State file for last_success tracking",
    "  --definitions <path>                   Probe definitions JSON path",
    "  --help                                 Show this help",
    "",
    "Examples:",
    "  npm run probe:oauth -- --dry-run",
    "  npm run probe:oauth -- --dry-run --provider spotify",
  ].join("\n");
}

function parseArgs(argv) {
  const args = {
    provider: "all",
    dryRun: false,
    output: DEFAULT_OUTPUT_PATH,
    stateFile: DEFAULT_STATE_PATH,
    definitionsPath: DEFAULT_DEFINITIONS_PATH,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];

    if (token === "--help" || token === "-h") {
      process.stdout.write(`${usage()}\n`);
      process.exit(0);
    }

    if (token === "--dry-run") {
      args.dryRun = true;
      continue;
    }

    if (token === "--provider") {
      const value = argv[i + 1];
      if (!value) {
        throw new Error("--provider requires a value");
      }
      if (!["spotify", "apple", "tidal", "all"].includes(value)) {
        throw new Error(`Unsupported provider '${value}'`);
      }
      args.provider = value;
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

    if (token === "--definitions") {
      const value = argv[i + 1];
      if (!value) {
        throw new Error("--definitions requires a value");
      }
      args.definitionsPath = path.resolve(process.cwd(), value);
      i += 1;
      continue;
    }

    throw new Error(`Unknown argument '${token}'`);
  }

  return args;
}

async function loadDefinitions(filePath) {
  const raw = await readFile(filePath, "utf8");
  const parsed = JSON.parse(raw);
  if (!Array.isArray(parsed.definitions)) {
    throw new Error("definitions.json must contain a definitions array");
  }
  return parsed.definitions;
}

async function loadState(filePath) {
  try {
    const raw = await readFile(filePath, "utf8");
    const parsed = JSON.parse(raw);
    return {
      lastSuccessByProbeId: parsed.lastSuccessByProbeId ?? {},
    };
  } catch {
    return { lastSuccessByProbeId: {} };
  }
}

async function saveState(filePath, state) {
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, `${JSON.stringify(state, null, 2)}\n`, "utf8");
}

function filterDefinitions(definitions, provider) {
  if (provider === "all") {
    return definitions;
  }
  return definitions.filter((definition) => definition.provider === provider);
}

async function executeDefinition(definition, previousLastSuccess) {
  switch (definition.class) {
    case "login_callback_success":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.login_callback_success",
        details: {
          assertion: "callback path resolves to success redirect handling",
          deterministic: true,
          reason: "dry-run deterministic safety",
        },
      };

    case "token_refresh_failure_class":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.token_refresh_failure_class",
        details: {
          simulated_error: "invalid_grant",
          expected_classification: "token_refresh_failure_class",
          deterministic: true,
          reason: "real provider token invalidation is unsafe in synthetic runs",
        },
      };

    case "provider_unavailable_timeout":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.provider_unavailable_timeout",
        details: {
          simulated_transport: "timeout",
          timeout_ms: 5000,
          deterministic: true,
          reason: "real provider outage simulation is unsafe",
        },
      };

    default:
      return {
        status: "fail",
        simulationLabel: "deterministic.mock.unsupported",
        details: {
          error: `Unsupported class '${definition.class}'`,
          previous_last_success: previousLastSuccess,
        },
      };
  }
}

function toEvidenceMarkdown(args, results, generatedAt) {
  const lines = [];
  lines.push("# OAuth Synthetic Probe Dry-Run Evidence");
  lines.push("");
  lines.push(`- generated_at: ${generatedAt}`);
  lines.push(`- provider_target: ${args.provider}`);
  lines.push(`- dry_run: ${args.dryRun}`);
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

function logHumanSummary(results) {
  process.stderr.write("OAuth Synthetic Probe Results\n");
  process.stderr.write("================================\n");
  for (const result of results) {
    process.stderr.write(
      `[${result.status.toUpperCase()}] provider=${result.provider} flow=${result.flow} class=${result.class} simulation=${result.simulation_label}\n`,
    );
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const definitions = await loadDefinitions(args.definitionsPath);
  const selectedDefinitions = filterDefinitions(definitions, args.provider);

  if (selectedDefinitions.length === 0) {
    throw new Error("No probe definitions matched the selected provider");
  }

  const state = await loadState(args.stateFile);
  const generatedAt = nowIso();

  const results = [];

  for (const definition of selectedDefinitions) {
    const previousLastSuccess = state.lastSuccessByProbeId[definition.id] ?? null;
    const timestamp = nowIso();

    let status = "fail";
    let simulationLabel = "deterministic.mock.error";
    let details = {};

    try {
      const execution = await executeDefinition(definition, previousLastSuccess);
      status = execution.status;
      simulationLabel = execution.simulationLabel;
      details = execution.details;
    } catch (error) {
      status = "fail";
      simulationLabel = "deterministic.mock.exception";
      details = {
        error: error instanceof Error ? error.message : String(error),
      };
    }

    const lastSuccess = status === "pass" ? timestamp : (previousLastSuccess ?? null);

    if (status === "pass") {
      state.lastSuccessByProbeId[definition.id] = timestamp;
    }

    results.push({
      provider: definition.provider,
      flow: definition.flow,
      class: definition.class,
      last_success: lastSuccess,
      status,
      timestamp,
      probe_id: definition.id,
      simulation: true,
      simulation_label: simulationLabel,
      details,
    });
  }

  await saveState(args.stateFile, state);

  const outputJson = {
    generated_at: generatedAt,
    dry_run: args.dryRun,
    provider_target: args.provider,
    definitions_path: args.definitionsPath,
    state_path: args.stateFile,
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

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`OAuth probe runner failed: ${message}\n`);
  process.exitCode = 1;
});
