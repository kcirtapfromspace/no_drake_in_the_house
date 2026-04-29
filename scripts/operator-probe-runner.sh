#!/usr/bin/env bash
# operator-probe-runner.sh - NOD-237 operator probe lane runner
#
# Executes a registered probe set against a target environment, writes
# bracketing operator_probe_run rows into audit_log, and posts the
# captured evidence as a comment on a Paperclip issue.
#
# Probe registry: scripts/operator-probes/<name>.sh
# Each probe MUST be self-contained, idempotent, and read + marker-write
# only -- no destructive prod operations are allowed in this lane.
#
# Required env:
#   DATABASE_URL                   libpq URL for the probe target
#   PAPERCLIP_API_URL              base URL of the Paperclip control plane
#   PAPERCLIP_API_KEY              API token with comment-create scope
#
# Required args / env:
#   PROBE_NAME                     name from scripts/operator-probes/
#                                  (e.g. audit-log-health)
#   TARGET_ISSUE                   Paperclip identifier (NOD-195) or UUID
#
# Optional env:
#   PROBE_ACTOR                    default: github-actions:operator-probe
#   PROBE_DRY_RUN                  when 'true', skip audit insert + post
#   PROBE_TARGET_ENV               label surfaced in evidence (e.g. prod)
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: PROBE_NAME=<name> TARGET_ISSUE=<NOD-XXX|uuid> \
       DATABASE_URL=... PAPERCLIP_API_URL=... PAPERCLIP_API_KEY=... \
       scripts/operator-probe-runner.sh

  PROBE_NAME      registered script under scripts/operator-probes/<name>.sh
  TARGET_ISSUE    Paperclip issue identifier or UUID to receive evidence
USAGE
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'operator-probe-runner: missing required command: %s\n' "$1" >&2
    exit 2
  fi
}

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    printf 'operator-probe-runner: %s is required\n' "$name" >&2
    usage >&2
    exit 2
  fi
}

require_cmd psql
require_cmd curl
require_cmd python3
require_cmd shasum
require_cmd jq

require_env DATABASE_URL
require_env PAPERCLIP_API_URL
require_env PAPERCLIP_API_KEY
require_env PROBE_NAME
require_env TARGET_ISSUE

PROBE_ACTOR="${PROBE_ACTOR:-github-actions:operator-probe}"
PROBE_DRY_RUN="${PROBE_DRY_RUN:-false}"
PROBE_TARGET_ENV="${PROBE_TARGET_ENV:-unknown}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
probe_path="${repo_root}/scripts/operator-probes/${PROBE_NAME}.sh"

if [[ ! -x "$probe_path" ]]; then
  if [[ -f "$probe_path" ]]; then
    chmod +x "$probe_path"
  else
    printf 'operator-probe-runner: probe not found: %s\n' "$probe_path" >&2
    printf 'available probes:\n' >&2
    find "${repo_root}/scripts/operator-probes" -maxdepth 1 -name '*.sh' \
      -exec basename {} .sh \; >&2 || true
    exit 2
  fi
fi

PROBE_RUN_ID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
PROBE_HASH="$(shasum -a 256 "$probe_path" | awk '{print $1}')"
RUN_STARTED_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT
output_file="${tmpdir}/probe-output.txt"

PSQL=(psql "$DATABASE_URL" --no-psqlrc --no-align --tuples-only --quiet
      -v ON_ERROR_STOP=1)

audit_insert() {
  local stage="$1"        # started | completed | failed
  local exit_code="$2"    # numeric or '' for started
  local action_name="operator_probe_run_${stage}"

  if [[ "$PROBE_DRY_RUN" == "true" ]]; then
    return 0
  fi

  "${PSQL[@]}" -c "INSERT INTO audit_log (
        user_id, action, event_type, severity, details, timestamp
      ) VALUES (
        NULL,
        '${action_name}',
        'system_event'::audit_event_type,
        'info'::audit_severity,
        jsonb_build_object(
          'probe_run_id', '${PROBE_RUN_ID}',
          'probe_name',  '${PROBE_NAME}',
          'probe_hash',  '${PROBE_HASH}',
          'actor',       '${PROBE_ACTOR}',
          'target_env',  '${PROBE_TARGET_ENV}',
          'target_issue','${TARGET_ISSUE}',
          'stage',       '${stage}',
          'exit_code',   ${exit_code:-null},
          'origin',      'operator_probe_lane'
        ),
        NOW()
      );" >/dev/null
}

post_comment() {
  local body_file="$1"
  if [[ "$PROBE_DRY_RUN" == "true" ]]; then
    printf 'operator-probe-runner: dry run -- skipping comment post\n' >&2
    return 0
  fi

  local payload="${tmpdir}/payload.json"
  jq -Rs --arg run "$PROBE_RUN_ID" --arg name "$PROBE_NAME" \
        '{body: .,
          metadata: {operator_probe_run_id: $run, probe_name: $name}}' \
    "$body_file" > "$payload"

  local url="${PAPERCLIP_API_URL%/}/api/issues/${TARGET_ISSUE}/comments"
  local http_code
  http_code=$(curl -sS -o "${tmpdir}/post.body" -w '%{http_code}' \
    -X POST "$url" \
    -H "Authorization: Bearer ${PAPERCLIP_API_KEY}" \
    -H 'Content-Type: application/json' \
    --data @"$payload")

  if [[ "$http_code" =~ ^2 ]]; then
    printf 'operator-probe-runner: posted comment to %s (HTTP %s)\n' \
      "$TARGET_ISSUE" "$http_code" >&2
  else
    printf 'operator-probe-runner: comment post failed HTTP %s\n' \
      "$http_code" >&2
    cat "${tmpdir}/post.body" >&2 || true
    return 1
  fi
}

build_evidence() {
  local exit_code="$1"
  local body="${tmpdir}/comment.md"
  {
    printf '## Operator probe evidence — %s\n\n' "$PROBE_NAME"
    printf '- run_id: `%s`\n' "$PROBE_RUN_ID"
    printf '- probe_hash: `%s`\n' "$PROBE_HASH"
    printf '- actor: `%s`\n' "$PROBE_ACTOR"
    printf '- target_env: `%s`\n' "$PROBE_TARGET_ENV"
    printf '- target_issue: `%s`\n' "$TARGET_ISSUE"
    printf '- run_started_utc: `%s`\n' "$RUN_STARTED_UTC"
    printf '- exit_code: `%s`\n' "$exit_code"
    printf '\n'
    printf '### Captured output\n\n'
    cat "$output_file"
    printf '\n'
    printf '_Posted automatically by `scripts/operator-probe-runner.sh` '
    printf 'via the operator probe lane (NOD-237)._\n'
  } > "$body"
  printf '%s' "$body"
}

# ---------- run ----------
printf 'operator-probe-runner: starting probe=%s run_id=%s target_issue=%s\n' \
  "$PROBE_NAME" "$PROBE_RUN_ID" "$TARGET_ISSUE" >&2

audit_insert started ''

set +e
PROBE_RUN_ID="$PROBE_RUN_ID" PROBE_ACTOR="$PROBE_ACTOR" \
  bash "$probe_path" >"$output_file" 2>&1
probe_exit=$?
set -e

if (( probe_exit == 0 )); then
  audit_insert completed "$probe_exit"
  body_file="$(build_evidence "$probe_exit")"
  post_comment "$body_file"
else
  audit_insert failed "$probe_exit"
  body_file="$(build_evidence "$probe_exit")"
  post_comment "$body_file" || true
fi

cat "$output_file"
exit "$probe_exit"
