-- 044_operator_probe_lane.sql
-- NOD-237: provision the restricted probe_runner role and document the
-- operator probe lane action names in audit_log.
--
-- The probe_runner role is the only credential surface allowed in CI for
-- prod-side probes. It can read everything required to inspect schema /
-- triggers and INSERT into audit_log only. It MUST NOT have UPDATE,
-- DELETE, TRUNCATE on audit_log or any other table.

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'probe_runner') THEN
    -- LOGIN role with no superuser, no createdb, no createrole.
    -- Password is rotated out-of-band; the migration only ensures the
    -- role exists with the correct privilege envelope.
    CREATE ROLE probe_runner LOGIN
      NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION
      CONNECTION LIMIT 4;
  END IF;
END
$$;

-- Database / schema connect. Use a DO block so CURRENT_DATABASE() is
-- resolvable in the GRANT statement.
DO $$
BEGIN
  EXECUTE format('GRANT CONNECT ON DATABASE %I TO probe_runner',
                 current_database());
END
$$;
GRANT USAGE ON SCHEMA public TO probe_runner;

-- Read everything needed for diagnostic SELECTs.
GRANT SELECT ON ALL TABLES IN SCHEMA public TO probe_runner;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
  GRANT SELECT ON TABLES TO probe_runner;

-- INSERT-only on audit_log so probes can land their marker rows. Explicit
-- REVOKE of UPDATE/DELETE/TRUNCATE keeps this lane append-only even if
-- someone later widens public defaults.
GRANT INSERT ON TABLE audit_log TO probe_runner;
REVOKE UPDATE, DELETE, TRUNCATE ON TABLE audit_log FROM probe_runner;

-- audit_event_type / audit_severity enum usage so probe inserts can cast
-- to system_event / info without elevated privileges.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'audit_event_type') THEN
    EXECUTE 'GRANT USAGE ON TYPE audit_event_type TO probe_runner';
  END IF;
  IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'audit_severity') THEN
    EXECUTE 'GRANT USAGE ON TYPE audit_severity TO probe_runner';
  END IF;
END
$$;

-- Document the registered operator-probe action names. These are
-- conventions, not enum values, but recording them in a comment makes
-- audit log archaeology obvious.
COMMENT ON TABLE audit_log IS
  'Append-only audit log. Operator probe lane (NOD-237) writes rows with '
  'action IN (''operator_probe_run_started'',''operator_probe_run_completed'','
  '''operator_probe_run_failed'',''audit_schema_smoke_probe'') under the '
  'system_event event_type. The probe_runner role has INSERT-only access.';
