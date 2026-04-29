#!/usr/bin/env bash
# audit-log-health - registered probe set for the operator probe lane.
#
# Validates the prod audit_log schema mode, OAuth audit trigger wiring,
# inserts an idempotent marker row, and reports counts plus MAX(timestamp).
# Composed of the standalone scripts/audit-log-schema-probe.sh script so
# the same probe is also pasteable into a Render shell as a fallback.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec bash "${repo_root}/scripts/audit-log-schema-probe.sh"
