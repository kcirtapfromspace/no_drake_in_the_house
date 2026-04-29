#!/usr/bin/env bash
set -euo pipefail

DB_URL="${DATABASE_URL:-${1:-}}"
if [[ -z "${DB_URL}" ]]; then
  echo "Usage: DATABASE_URL=postgres://... $0"
  echo "   or: $0 postgres://..."
  exit 1
fi

psql_cmd=(psql "${DB_URL}" -v ON_ERROR_STOP=1)

echo "== audit_log schema mode =="
schema_mode="$("${psql_cmd[@]}" -Atc "
SELECT
  CASE
    WHEN EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = 'public'
        AND table_name = 'audit_log'
        AND column_name = 'actor_user_id'
    ) THEN 'legacy'
    WHEN EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = 'public'
        AND table_name = 'audit_log'
        AND column_name = 'user_id'
    ) THEN 'migrated'
    ELSE 'unknown'
  END AS schema_mode
;")"
echo "schema_mode=${schema_mode}"

echo
echo "== audit_log relevant columns =="
"${psql_cmd[@]}" -c "
SELECT
  column_name,
  data_type,
  is_nullable
FROM information_schema.columns
WHERE table_schema = 'public'
  AND table_name = 'audit_log'
  AND column_name IN (
    'actor_user_id',
    'user_id',
    'subject_type',
    'old_subject_type',
    'subject_id',
    'old_subject_id',
    'created_at',
    'timestamp',
    'details'
  )
ORDER BY column_name;
"

echo
echo "== OAuth audit trigger wiring =="
"${psql_cmd[@]}" -c "
SELECT
  c.relname AS table_name,
  tg.tgname AS trigger_name,
  p.proname AS function_name
FROM pg_trigger tg
JOIN pg_class c
  ON c.oid = tg.tgrelid
JOIN pg_proc p
  ON p.oid = tg.tgfoid
JOIN pg_namespace n
  ON n.oid = c.relnamespace
WHERE n.nspname = 'public'
  AND c.relname IN ('oauth_accounts', 'account_merges')
  AND NOT tg.tgisinternal
ORDER BY c.relname, tg.tgname;
"

echo
echo "== Deterministic audit_log insert smoke =="
before_count="$("${psql_cmd[@]}" -Atc "SELECT COUNT(*) FROM audit_log WHERE action = 'audit_schema_smoke_probe';")"
probe_subject_id="probe-$(date -u +%Y%m%dT%H%M%SZ)-${RANDOM}"

if [[ "${schema_mode}" == "legacy" ]]; then
  "${psql_cmd[@]}" -c "
    INSERT INTO audit_log (
      actor_user_id, action, subject_type, subject_id, after_state, created_at
    )
    VALUES (
      NULL,
      'audit_schema_smoke_probe',
      'audit_probe',
      '${probe_subject_id}',
      jsonb_build_object('schema_mode', 'legacy'),
      NOW()
    );
  "
  latest_probe_ts="$("${psql_cmd[@]}" -Atc "
    SELECT to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
    FROM audit_log
    WHERE action = 'audit_schema_smoke_probe'
      AND subject_id = '${probe_subject_id}'
    ORDER BY created_at DESC
    LIMIT 1;
  ")"
elif [[ "${schema_mode}" == "migrated" ]]; then
  "${psql_cmd[@]}" -c "
    INSERT INTO audit_log (
      user_id, action, old_subject_type, old_subject_id, details, timestamp
    )
    VALUES (
      NULL,
      'audit_schema_smoke_probe',
      'audit_probe',
      '${probe_subject_id}',
      jsonb_build_object('schema_mode', 'migrated'),
      NOW()
    );
  "
  latest_probe_ts="$("${psql_cmd[@]}" -Atc "
    SELECT to_char(timestamp AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
    FROM audit_log
    WHERE action = 'audit_schema_smoke_probe'
      AND old_subject_id = '${probe_subject_id}'
    ORDER BY timestamp DESC
    LIMIT 1;
  ")"
else
  echo "Unknown audit_log schema mode; aborting smoke insert."
  exit 2
fi

after_count="$("${psql_cmd[@]}" -Atc "SELECT COUNT(*) FROM audit_log WHERE action = 'audit_schema_smoke_probe';")"
expected_count=$((before_count + 1))

echo "before_count=${before_count}"
echo "after_count=${after_count}"
echo "expected_after_count=${expected_count}"
echo "probe_subject_id=${probe_subject_id}"
echo "latest_probe_timestamp_utc=${latest_probe_ts}"

if [[ "${after_count}" -ne "${expected_count}" ]]; then
  echo "ERROR: audit_log count did not increase by 1"
  exit 3
fi

echo "Smoke check passed."

echo
echo "== OAuth trigger write smoke (migrated schema) =="
oauth_accounts_exists="$("${psql_cmd[@]}" -Atc "SELECT to_regclass('public.oauth_accounts') IS NOT NULL;")"
oauth_trigger_exists="$("${psql_cmd[@]}" -Atc "
SELECT EXISTS (
  SELECT 1
  FROM pg_trigger tg
  JOIN pg_class c
    ON c.oid = tg.tgrelid
  JOIN pg_namespace n
    ON n.oid = c.relnamespace
  WHERE n.nspname = 'public'
    AND c.relname = 'oauth_accounts'
    AND tg.tgname = 'oauth_account_audit_trigger'
    AND NOT tg.tgisinternal
);
")"

if [[ "${schema_mode}" != "migrated" ]]; then
  echo "Skipped: schema_mode=${schema_mode} (requires migrated schema)."
elif [[ "${oauth_accounts_exists}" != "t" || "${oauth_trigger_exists}" != "t" ]]; then
  echo "Skipped: oauth_accounts table or oauth_account_audit_trigger not present."
else
  probe_suffix="$(date -u +%Y%m%dT%H%M%SZ)-${RANDOM}"
  probe_email="audit-probe-${probe_suffix}@example.com"
  probe_provider_user_id="audit-probe-provider-${probe_suffix}"

  before_oauth_count="$("${psql_cmd[@]}" -Atc "
    SELECT COUNT(*)
    FROM audit_log
    WHERE action = 'oauth_account_created'
      AND after_state->>'provider_user_id' = '${probe_provider_user_id}';
  ")"

  probe_user_id="$("${psql_cmd[@]}" -Atc "
    INSERT INTO users (email, password_hash)
    VALUES ('${probe_email}', 'probe_hash')
    RETURNING id;
  ")"
  probe_user_id="$(printf '%s\n' "${probe_user_id}" | head -n 1)"

  probe_oauth_id="$("${psql_cmd[@]}" -Atc "
    INSERT INTO oauth_accounts (
      user_id, provider, provider_user_id, email, display_name
    )
    VALUES (
      '${probe_user_id}',
      'spotify',
      '${probe_provider_user_id}',
      '${probe_email}',
      'Audit Probe'
    )
    RETURNING id;
  ")"
  probe_oauth_id="$(printf '%s\n' "${probe_oauth_id}" | head -n 1)"

  after_oauth_count="$("${psql_cmd[@]}" -Atc "
    SELECT COUNT(*)
    FROM audit_log
    WHERE action = 'oauth_account_created'
      AND after_state->>'provider_user_id' = '${probe_provider_user_id}';
  ")"
  expected_oauth_count=$((before_oauth_count + 1))

  latest_oauth_row="$("${psql_cmd[@]}" -Atc "
    SELECT
      COALESCE(user_id::text, ''),
      COALESCE(old_subject_type, ''),
      COALESCE(old_subject_id, ''),
      to_char(timestamp AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
    FROM audit_log
    WHERE action = 'oauth_account_created'
      AND after_state->>'provider_user_id' = '${probe_provider_user_id}'
    ORDER BY timestamp DESC
    LIMIT 1;
  ")"
  IFS='|' read -r logged_user_id logged_subject_type logged_subject_id logged_timestamp <<< "${latest_oauth_row}"

  echo "oauth_probe_user_id=${probe_user_id}"
  echo "oauth_probe_account_id=${probe_oauth_id}"
  echo "oauth_before_count=${before_oauth_count}"
  echo "oauth_after_count=${after_oauth_count}"
  echo "oauth_expected_after_count=${expected_oauth_count}"
  echo "oauth_logged_user_id=${logged_user_id}"
  echo "oauth_logged_subject_type=${logged_subject_type}"
  echo "oauth_logged_subject_id=${logged_subject_id}"
  echo "oauth_logged_timestamp_utc=${logged_timestamp}"

  if [[ "${after_oauth_count}" -ne "${expected_oauth_count}" ]]; then
    echo "ERROR: oauth trigger audit count did not increase by 1"
    exit 4
  fi

  if [[ "${logged_user_id}" != "${probe_user_id}" ]]; then
    echo "ERROR: oauth trigger logged user_id does not match probe user"
    exit 5
  fi

  if [[ "${logged_subject_type}" != "oauth_account" ]]; then
    echo "ERROR: oauth trigger logged old_subject_type is not oauth_account"
    exit 6
  fi

  if [[ "${logged_subject_id}" != "${probe_oauth_id}" ]]; then
    echo "ERROR: oauth trigger logged old_subject_id does not match inserted oauth account id"
    exit 7
  fi

  echo "OAuth trigger smoke check passed."
fi
