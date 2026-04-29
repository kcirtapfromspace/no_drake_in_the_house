# PR Manager — stale heartbeat queue drain runbook

## When to use this

Symptoms that mean PR Manager (or any `codex_local` agent) has a wedged queue:

- `GET /api/companies/{companyId}/heartbeat-runs?agentId=...` returns rows with `status=queued` and `created_at` older than ~10 minutes.
- Activity feed for the agent shows wakes "received" but never "started".
- Server log shows repeating `periodic heartbeat recovery failed` errors for the company.
- PR Manager `lastHeartbeatAt` is stale or `agents.status` is stuck at `error`.

Root cause for the original incident on 2026-04-28: every 30s heartbeat recovery cycle aborted on a unique-constraint violation (`issues_open_routine_execution_uq`) because a paused routine had 61 open backlog issues sharing its `origin_id`. Recovery couldn't transition stranded agents out of `error`, and a server restart at 18:56:17 UTC killed PR Manager's codex child process. Platform fix landed in [NOD-220](/NOD/issues/NOD-220); this runbook covers the operator-side queue drain.

## 1. Detect

```sql
-- run against the embedded Postgres: postgres://paperclip:paperclip@127.0.0.1:54329/paperclip
SELECT status, COUNT(*),
       to_char(MIN(created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS oldest_utc,
       to_char(MAX(created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS newest_utc
FROM heartbeat_runs
WHERE agent_id = '<pr-manager-agent-id>'
  AND created_at > NOW() - INTERVAL '24 hours'
GROUP BY status
ORDER BY status;
```

Stale = `status='queued'` AND `created_at < NOW() - INTERVAL '10 minutes'`.

Cross-check that the agent actually has *no live* run before draining:

```sql
SELECT id, started_at, process_pid
FROM heartbeat_runs
WHERE agent_id = '<pr-manager-agent-id>' AND status = 'running';
```

If there is a `running` row, verify the pid is alive (`ps -p <pid>`). Do **not** touch a row whose pid is alive — let it run to terminal.

Also tail the server log for resolved-or-still-failing recovery loop:

```bash
tail -n 200 ~/.paperclip/instances/default/logs/server.log | grep -iE "periodic heartbeat recovery failed|process_lost|Process lost"
```

## 2. Try the API first

Board access can use the cancel endpoint:

```bash
curl -X POST -H "Authorization: Bearer $BOARD_KEY" \
     -H "X-Paperclip-Run-Id: $RUN_ID" \
     -H "Content-Type: application/json" \
     "$PAPERCLIP_API_URL/api/heartbeat-runs/<runId>/cancel" \
     -d '{"reason":"stale-queue drain"}'
```

Agent JWTs (Release Engineer, CTO, etc.) get `403 Board access required` — `assertBoard` is hardcoded. If you can't get a board key, fall back to step 3.

## 3. DB-level drain (fallback)

Mirrors `cancelRunInternal` in `server/src/services/heartbeat.ts` for the no-process case (queued runs have no pid to terminate). One transaction:

```sql
BEGIN;

CREATE TEMP TABLE drain_targets AS
SELECT id, wakeup_request_id, company_id, agent_id
FROM heartbeat_runs
WHERE agent_id = '<pr-manager-agent-id>'
  AND status = 'queued'
  AND created_at < NOW() - INTERVAL '10 minutes';

-- 1) cancel the heartbeat_runs rows (only the still-queued ones)
UPDATE heartbeat_runs hr
SET status = 'cancelled',
    finished_at = NOW(),
    error = '<reason — link the issue or incident>',
    error_code = 'cancelled',
    result_json = COALESCE(hr.result_json, '{}'::jsonb) || jsonb_build_object(
      'stopReason','cancelled',
      'timeoutFired',false,
      'timeoutSource','config',
      'timeoutConfigured',false,
      'effectiveTimeoutSec',0
    ),
    updated_at = NOW()
FROM drain_targets t
WHERE hr.id = t.id AND hr.status = 'queued';

-- 2) cancel the linked agent_wakeup_requests rows
UPDATE agent_wakeup_requests w
SET status = 'cancelled',
    finished_at = NOW(),
    error = 'companion run cancelled',
    updated_at = NOW()
FROM drain_targets t
WHERE w.id = t.wakeup_request_id
  AND w.status NOT IN ('cancelled','completed','failed','skipped');

-- 3) write lifecycle event so the activity feed reflects the cancel
INSERT INTO heartbeat_run_events
  (company_id, run_id, agent_id, seq, event_type, stream, level, message, payload, created_at)
SELECT t.company_id, t.id, t.agent_id, 1,
       'lifecycle','system','warn','run cancelled',
       jsonb_build_object('reason','<reason>','operator','<who you are>'),
       NOW()
FROM drain_targets t;

COMMIT;
```

**Do not** drain `running` rows this way — that path requires terminating the live process via `terminateHeartbeatRunProcess`. Use the API cancel route or restart the supervisor instead.

## 4. Verify

```sql
SELECT status, COUNT(*) FROM heartbeat_runs
WHERE agent_id = '<pr-manager-agent-id>'
  AND created_at > NOW() - INTERVAL '24 hours'
GROUP BY status;

SELECT status, last_heartbeat_at FROM agents WHERE id = '<pr-manager-agent-id>';
```

Acceptance:

- 0 rows where `status='queued'` AND `created_at < NOW() - INTERVAL '10 minutes'`
- Agent `status` is `idle` or `running` (not `error`)
- Next automation tick produces a fresh `succeeded` run within ~5 minutes (compare `started_at`/`finished_at`)
- Server log has no new `process_lost` for that agent and no new `periodic heartbeat recovery failed`

## 5. Report

Post the before/after counts on the originating recovery issue (e.g., NOD-217-style ticket) and link the platform fix that unwedged the recovery loop (NOD-220 in this incident). Update this runbook if the schema or service contract changes.

## Related

- Platform fix: [NOD-220](/NOD/issues/NOD-220) — heartbeat-recovery insert is now idempotent on `issues_open_routine_execution_uq`
- Earlier symptom-only ticket: [NOD-215](/NOD/issues/NOD-215)
- Stabilization ticket this runbook resolves: [NOD-227](/NOD/issues/NOD-227)
- Code: `server/src/services/heartbeat.ts::cancelRunInternal` (the canonical cancel flow this DB drain mirrors)
