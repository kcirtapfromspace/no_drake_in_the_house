#!/usr/bin/env bash
# audit-log-schema-probe.sh - NOD-195 / NOD-237 audit_log health probe
#
# Read + idempotent marker insert against audit_log. Designed to run
# under the operator probe lane (see docs/operator-probe-lane.md), but
# also pasteable into a Render shell.
#
# Inputs (env):
#   DATABASE_URL          required, libpq URL
#   PROBE_RUN_ID          optional, surfaced into the marker row details
#                         (defaults to a fresh uuid)
#   PROBE_ACTOR           optional, who/what is running this probe
#                         (defaults to ${USER:-operator-probe})
#   MARKER_ACTION         optional, audit_log.action for the marker insert
#                         (defaults to audit_schema_smoke_probe)
#
# Output: a single ```text block on stdout suitable for pasting verbatim
# into a Paperclip comment. Exits non-zero only on probe execution
# failure; schema-mode mismatches are reported, not failed.
set -euo pipefail

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'audit-log-schema-probe: missing required command: %s\n' "$1" >&2
    exit 2
  fi
}

require_cmd psql

if [[ -z "${DATABASE_URL:-}" ]]; then
  printf 'audit-log-schema-probe: DATABASE_URL is required\n' >&2
  exit 2
fi

PROBE_RUN_ID="${PROBE_RUN_ID:-$(python3 -c 'import uuid; print(uuid.uuid4())')}"
PROBE_ACTOR="${PROBE_ACTOR:-${USER:-operator-probe}}"
MARKER_ACTION="${MARKER_ACTION:-audit_schema_smoke_probe}"

PSQL=(psql "$DATABASE_URL" --no-psqlrc --no-align --tuples-only --quiet
      -v ON_ERROR_STOP=1)

scalar() {
  "${PSQL[@]}" -c "$1" | tr -d '[:space:]'
}

block() {
  "${PSQL[@]}" --pset=format=aligned -c "$1"
}

# ---------- schema mode ----------
has_timestamp_col=$(scalar "SELECT EXISTS (
  SELECT 1 FROM information_schema.columns
  WHERE table_name='audit_log' AND column_name='timestamp'
);")
has_user_id_col=$(scalar "SELECT EXISTS (
  SELECT 1 FROM information_schema.columns
  WHERE table_name='audit_log' AND column_name='user_id'
);")
has_actor_user_id_col=$(scalar "SELECT EXISTS (
  SELECT 1 FROM information_schema.columns
  WHERE table_name='audit_log' AND column_name='actor_user_id'
);")

if [[ "$has_timestamp_col" == "t" && "$has_user_id_col" == "t" ]]; then
  schema_mode="migrated"
elif [[ "$has_actor_user_id_col" == "t" ]]; then
  schema_mode="legacy"
else
  schema_mode="unknown"
fi

ts_col="timestamp"
[[ "$schema_mode" == "legacy" ]] && ts_col="created_at"

run_started_utc=$(scalar "SELECT to_char(NOW() AT TIME ZONE 'UTC',
  'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"');")

# ---------- output header ----------
printf '```text\n'
printf '## audit-log-schema-probe\n'
printf 'run_id=%s\n' "$PROBE_RUN_ID"
printf 'actor=%s\n' "$PROBE_ACTOR"
printf 'run_started_utc=%s\n' "$run_started_utc"
printf 'schema_mode=%s\n' "$schema_mode"
printf 'marker_action=%s\n' "$MARKER_ACTION"
printf -- '----- trigger wiring -----\n'

# ---------- trigger wiring ----------
block "SELECT tgname AS trigger,
              c.relname AS table,
              CASE WHEN tgenabled = 'D' THEN 'disabled' ELSE 'enabled' END AS state
       FROM pg_trigger t
       JOIN pg_class c ON c.oid = t.tgrelid
       WHERE NOT t.tgisinternal
         AND tgname IN ('oauth_account_audit_trigger',
                        'account_merge_audit_trigger')
       ORDER BY tgname;"

# ---------- marker insert ----------
printf -- '----- marker insert -----\n'

before_count=$(scalar "SELECT COUNT(*) FROM audit_log
  WHERE action = '${MARKER_ACTION}';")

if [[ "$schema_mode" == "migrated" ]]; then
  block "INSERT INTO audit_log (
            user_id, action, event_type, severity, details, ${ts_col}
         ) VALUES (
            NULL,
            '${MARKER_ACTION}',
            'system_event'::audit_event_type,
            'info'::audit_severity,
            jsonb_build_object(
              'probe_run_id', '${PROBE_RUN_ID}',
              'actor', '${PROBE_ACTOR}',
              'origin', 'operator_probe_lane'
            ),
            NOW()
         )
         RETURNING id, ${ts_col};"
elif [[ "$schema_mode" == "legacy" ]]; then
  block "INSERT INTO audit_log (
            actor_user_id, action, subject_type, subject_id,
            after_state, ${ts_col}
         ) VALUES (
            NULL,
            '${MARKER_ACTION}',
            'operator_probe',
            '${PROBE_RUN_ID}',
            jsonb_build_object(
              'actor', '${PROBE_ACTOR}',
              'origin', 'operator_probe_lane'
            ),
            NOW()
         )
         RETURNING id, ${ts_col};"
else
  printf 'schema_mode=unknown -- skipping marker insert\n'
fi

after_count=$(scalar "SELECT COUNT(*) FROM audit_log
  WHERE action = '${MARKER_ACTION}';")

# ---------- counts and recency ----------
total_count=$(scalar "SELECT COUNT(*) FROM audit_log;")
max_ts=$(scalar "SELECT to_char(MAX(${ts_col}) AT TIME ZONE 'UTC',
  'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') FROM audit_log;")
recent_marker_count=$(scalar "SELECT COUNT(*) FROM audit_log
  WHERE action = '${MARKER_ACTION}'
    AND ${ts_col} > NOW() - INTERVAL '10 minutes';")

printf -- '----- counts -----\n'
printf 'audit_log_total=%s\n' "$total_count"
printf 'marker_before=%s\n' "$before_count"
printf 'marker_after=%s\n' "$after_count"
printf 'marker_recent_10m=%s\n' "$recent_marker_count"
printf 'audit_log_max_ts_utc=%s\n' "$max_ts"
printf '```\n'

# ---------- correctness assertion ----------
if [[ "$schema_mode" == "migrated" || "$schema_mode" == "legacy" ]]; then
  if (( after_count <= before_count )); then
    printf 'audit-log-schema-probe: marker insert did not increase row count (%s -> %s)\n' \
      "$before_count" "$after_count" >&2
    exit 1
  fi
  if (( recent_marker_count < 1 )); then
    printf 'audit-log-schema-probe: no marker row visible within 10-minute window\n' >&2
    exit 1
  fi
fi
