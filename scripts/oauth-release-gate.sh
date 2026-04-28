#!/usr/bin/env bash
# oauth-release-gate.sh - NOD-164 SLO contract release gate
# Evaluates OAuth provider metrics against SLO thresholds.
# Fails closed: any parse error, missing data, or P1 violation blocks merge.
#
# Usage: ./scripts/oauth-release-gate.sh <metrics.json>

set -euo pipefail

METRICS_FILE="${1:-}"

if [ -z "$METRICS_FILE" ]; then
  echo "FAIL-CLOSED: No metrics file provided."
  echo "Usage: $0 <metrics.json>"
  exit 1
fi

if [ ! -f "$METRICS_FILE" ]; then
  echo "FAIL-CLOSED: Metrics file not found: $METRICS_FILE"
  exit 1
fi

export METRICS_FILE_PATH="$METRICS_FILE"

python3 << '___PYTHON_GATE___'
import json
import sys
import os

metrics_file = os.environ.get("METRICS_FILE_PATH")

PROVIDER_RULES = [
    ("OAUTH-ERR-001", "error_rate",              "gt",  0.05,  "5min",   "P1"),
    ("OAUTH-ERR-002", "failed_auth_count",        "gt",  10,    "5min",   "P1"),
    ("OAUTH-ERR-004", "failed_provider_streak",   "gte", 3,     "10min",  "P1"),
    ("OAUTH-ERR-005", "rate_limited_duration_min", "gt",  15,    "15min",  "P1"),
    ("OAUTH-LAT-001", "callback_p95_ms",          "gt",  3000,  "5min",   "P2"),
    ("OAUTH-LAT-002", "refresh_p95_ms",           "gt",  2000,  "5min",   "P2"),
    ("OAUTH-LAT-003", "probe_p95_ms",             "gt",  5000,  "5min",   "P2"),
    ("OAUTH-PRB-001", "probe_consecutive_fail",   "gte", 2,     "sched", "P2"),
    ("OAUTH-PRB-002", "probe_consecutive_fail",   "gte", 3,     "sched", "P1"),
    ("OAUTH-BUD-001", "callback_budget_burn",     "gt",  6.0,   "1h",    "P1"),
    ("OAUTH-BUD-002", "refresh_budget_burn",      "gt",  6.0,   "1h",    "P1"),
]

GLOBAL_RULES = [
    ("OAUTH-ERR-003", "failed_system_count", "gt", 1, "1min", "P1"),
]

try:
    with open(metrics_file, "r") as f:
        data = json.load(f)
except json.JSONDecodeError as e:
    print(f"FAIL-CLOSED: Invalid JSON in {metrics_file}: {e}")
    sys.exit(1)
except Exception as e:
    print(f"FAIL-CLOSED: Cannot read {metrics_file}: {e}")
    sys.exit(1)

if "providers" not in data:
    print("FAIL-CLOSED: Missing required key 'providers' in metrics JSON.")
    sys.exit(1)

if "global" not in data:
    print("FAIL-CLOSED: Missing required key 'global' in metrics JSON.")
    sys.exit(1)

violations = []
warnings = []


def compare(value, comparator, threshold):
    if comparator == "gt":
        return value > threshold
    elif comparator == "gte":
        return value >= threshold
    return False


providers = data["providers"]
for provider_name, provider_metrics in providers.items():
    for (code, metric_key, comparator, threshold, window, priority) in PROVIDER_RULES:
        if metric_key not in provider_metrics:
            violations.append((code, "P1", provider_name, metric_key, "MISSING",
                               threshold, "fail-closed: metric missing"))
            continue
        value = provider_metrics[metric_key]
        if compare(value, comparator, threshold):
            entry = (code, priority, provider_name, metric_key, value, threshold, window)
            if priority == "P1":
                violations.append(entry)
            else:
                warnings.append(entry)

global_metrics = data["global"]
for (code, metric_key, comparator, threshold, window, priority) in GLOBAL_RULES:
    if metric_key not in global_metrics:
        violations.append((code, "P1", "global", metric_key, "MISSING",
                           threshold, "fail-closed: metric missing"))
        continue
    value = global_metrics[metric_key]
    if compare(value, comparator, threshold):
        entry = (code, priority, "global", metric_key, value, threshold, window)
        if priority == "P1":
            violations.append(entry)
        else:
            warnings.append(entry)

sep = "=" * 72
dash = "-" * 72

print(sep)
print("  OAuth Release Gate - NOD-164 SLO Contract Evaluation")
print(sep)
print()

timestamp = data.get("timestamp", "unknown")
window = data.get("window", "unknown")
provider_list = ", ".join(providers.keys())
print(f"  Timestamp : {timestamp}")
print(f"  Window    : {window}")
print(f"  Providers : {provider_list}")
print()

if violations:
    print(dash)
    print("  P1 VIOLATIONS (merge blocked)")
    print(dash)
    for v in violations:
        code, pri, scope, metric, value, thresh, extra = v
        print(f"  [{code}] {pri}  {scope}/{metric} = {value}  (threshold: {thresh}, {extra})")
    print()

if warnings:
    print(dash)
    print("  P2 WARNINGS (non-blocking)")
    print(dash)
    for w in warnings:
        code, pri, scope, metric, value, thresh, extra = w
        print(f"  [{code}] {pri}  {scope}/{metric} = {value}  (threshold: {thresh}, {extra})")
    print()

if not violations and not warnings:
    print("  All SLO thresholds within limits.")
    print()

print(dash)

if violations:
    print("  RESULT: BLOCKED - P1 violations detected")
    print(f"  Total P1 violations : {len(violations)}")
    print(f"  Total P2 warnings   : {len(warnings)}")
    print(sep)
    sys.exit(1)
elif warnings:
    print("  RESULT: PASS (with warnings)")
    print(f"  Total P2 warnings   : {len(warnings)}")
    print(sep)
    sys.exit(0)
else:
    print("  RESULT: PASS")
    print(sep)
    sys.exit(0)
___PYTHON_GATE___
