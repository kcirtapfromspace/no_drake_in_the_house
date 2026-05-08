# Merge Event Automation Contract

Event-driven wakeup contract for permission-gated merge lanes. Replaces
heartbeat polling on blocked-merge issues with a deterministic transition
triggered by the upstream merge event.

Tracking: NOD-397 (this contract), NOD-394 (escalation policy this extends),
NOD-378 / NOD-363 / NOD-366 (pilot chain), NOD-347 (no-delta silence).

## Why this exists

The escalation policy (NOD-394) defines what happens up to the moment a
maintainer merges. Everything after the merge — clearing the blocker,
recording the SHA, waking QA — was still hand-driven through heartbeat
polling. That made the unblock latency a function of how often the Release
Engineer woke up, not how fast the merge actually landed.

This contract pins the post-merge half of the state machine to the merge
event itself, so the lane drains within one event cycle of the upstream
action.

## Scope

In scope:

- Detecting `pull_request.closed` with `merged=true` for any PR URL tracked
  on a `blocked` lane carrying the permission-gated merge label.
- Posting a merge-evidence comment to the maintainer task and to every
  downstream blocker the PR is recorded against.
- Transitioning the maintainer task to `done`, the merge-blocked lane to
  `in_progress` (release-side runtime proof remains owed), and any
  blocker-link in `blocks:` to cleared.
- Waking the Release and QA assignees on the cleared lanes.

Out of scope:

- Code deploy automation. This is workflow state automation only.
- Detecting CI failure on the merged commit. Post-merge proof remains
  Release Engineer responsibility.
- Auto-closing QA issues. QA still verifies on the live environment.

## Event source

Primary: GitHub webhook on the upstream repo, event `pull_request`, action
`closed`, with payload field `pull_request.merged == true`.

Fallback: a polling shim (see `scripts/release/merge_event_poller.py`) that
queries `gh pr view <n> --json state,mergedAt,mergedBy,mergeCommit` for every
tracked PR URL on every `blocked` issue with the permission-gated merge
escalation marker. Used until the webhook is wired in the control plane and
as a recovery path if a webhook delivery is missed.

Either source produces the same canonical event for the rest of the
pipeline:

```json
{
  "type": "pr.merged",
  "repo": "<owner>/<repo>",
  "number": 5402,
  "url": "https://github.com/<owner>/<repo>/pull/5402",
  "head_sha": "<head sha at merge time>",
  "merge_commit_sha": "<squash commit sha>",
  "merged_at": "<rfc3339>",
  "merged_by": "<github login>"
}
```

`merge_commit_sha` is the canonical evidence value. `head_sha` is captured
because some downstream systems index by it.

## Idempotency

Every transition this contract performs is keyed by:

```
merge-event:<repo>:<number>:<merge_commit_sha>
```

The handler MUST:

1. Look up the key in a persistent store (issue-thread label, dedupe table,
   or a comment containing the key — any durable surface in the control
   plane).
2. If the key is present, no-op the entire transition. No comments, no
   status changes, no wake events.
3. Otherwise apply the full transition atomically (all-or-nothing) and stamp
   the key on completion.

This guarantees replay-safety against:

- Duplicate webhook deliveries (GitHub retries on non-2xx).
- Webhook + polling shim racing during cutover.
- Operator manually re-firing the event after a failed first attempt.

A re-merge of a different commit (rare; only via revert + re-merge) produces
a different `merge_commit_sha` and is therefore a new event. That is correct
behavior — the new SHA is new evidence.

## State transitions

For each merge event, the handler walks the linked-issue graph rooted at
the maintainer task and applies the transitions below. All updates go
through the standard issue API; no direct DB writes.

| Linked issue role | Status before | Status after | Comment posted |
|---|---|---|---|
| Maintainer task (the `Maintainer merge PR #<n>` issue) | `blocked` | `done` | merge-evidence comment |
| Merge-blocked lane (issue whose work is unblocked by this PR) | `blocked` | `in_progress` | merge-evidence comment + reminder that runtime proof is still owed |
| Downstream QA lane (issue blocked by the merge-blocked lane via `blocks:`) | `blocked` | `in_progress` | merge-evidence comment + "wait for PostMerge_Proof before starting" |
| Any other inbound mention of the PR URL | unchanged | unchanged | merge-evidence comment only |

Wake fan-out:

- Maintainer task: no wake (it is `done`).
- Merge-blocked lane: wake the Release Engineer assignee.
- Downstream QA lane: wake the QA Engineer assignee.

Wakes are explicit, deterministic, and one-shot. No retries from the
event handler — if a wake fails, the next polling-shim cycle will retry the
whole event by virtue of the dedupe key not having been stamped.

## Comment template (merge-evidence)

Posted on every linked issue listed above. Outer fence is four backticks so
the inner triple-fence shell block renders correctly when copied verbatim.

````markdown
## Merged

PR: <url>
Merged at: <merged_at>
Merged by: <merged_by>
Merge SHA: `<merge_commit_sha>`

Triggered automatically by merge-event automation
(docs/release/merge-event-automation.md).

Idempotency key: `merge-event:<repo>:<number>:<merge_commit_sha>`
````

The merge-blocked lane comment additionally appends:

```
Next: Release Engineer publishes PostMerge_Proof (deploy + smoke). QA lane
remains gated on that proof per docs/release/permission-gated-merge-escalation.md.
```

The downstream QA lane comment additionally appends:

```
Wait for PostMerge_Proof on the upstream lane before starting verification.
```

## Pilot chain — NOD-378 / NOD-363 / NOD-366

The first lane this contract pilots against. Mapping below; values filled in
once PR #5402 actually merges.

| Role | Issue | Pre-event status | Post-event status |
|---|---|---|---|
| Maintainer task | NOD-378 | `blocked` | `done` |
| Merge-blocked lane | NOD-363 | `blocked` | `in_progress` |
| Downstream QA lane | NOD-366 | `blocked` | `in_progress` |

Acceptance for the pilot:

1. A single merge event drives all three rows above without further agent
   intervention.
2. Each row receives exactly one merge-evidence comment.
3. Re-firing the same event (manual or duplicate webhook) produces zero
   additional comments and zero status flips.
4. QA wake arrives within one event cycle (≤ shim poll interval, or webhook
   delivery latency).

## Replay-safety demonstration

The reference shim at `scripts/release/merge_event_poller.py` ships with a
`--self-test` mode that:

1. Constructs a synthetic merge event for a fixture PR.
2. Runs the handler logic in dry-run twice, asserting:
   - First run reports three transitions and three comments planned.
   - Second run reports zero transitions and zero comments planned (key
     already stamped).
3. Exits non-zero if either assertion fails.

`make release-merge-event-self-test` wraps the invocation. Running the
self-test is the contractual proof of replay-safety for this lane. The
GitHub Actions workflow `.github/workflows/release-merge-event-contract.yml`
runs it on every PR that touches the contract files (this doc, the shim,
the workflow, or the Makefile target), so contract regressions block merge.

## Live dry-run against the pilot chain

To verify the contract against real issue and PR state without writing
anything, run:

```bash
python3 scripts/release/merge_event_poller.py --from-paperclip NOD-378
```

The shim will:

1. Fetch the maintainer-task issue from the Paperclip API.
2. Walk its `relatedWork.outbound` to find merge-blocked lanes, then walk
   each merge-blocked lane's `blocks:` to find QA-downstream lanes.
3. Extract the PR URL from the maintainer-task description and call
   `gh pr view` to get live PR state.
4. If the PR is not merged: print "no event to fire" and exit 0.
5. If the PR is merged: assemble the canonical event and print the dry-run
   plan (transitions, wakes, idempotency key) — without any writes.

Operational use: run this from a routine on a tight cadence as the
fallback for the missing webhook subscription. The dry-run output is the
input the control-plane handler will eventually consume.

## Failure modes

- **Webhook lost.** Polling shim runs on a 60s cadence and re-fires the
  event. Idempotency key prevents double-application.
- **Issue API write fails mid-fan-out.** Handler does not stamp the
  idempotency key on partial failure; next event cycle replays the whole
  fan-out. Per-row API writes are individually idempotent (skip if the
  evidence comment for this key already exists on the row).
- **PR re-opened after merge=false event.** GitHub does not fire a `merged`
  event for non-merge closures, so this contract is a no-op for them. The
  escalation policy still applies: the lane stays `blocked`.
- **Squash commit force-pushed away.** Out of scope. If a maintainer rewrites
  history post-merge, the recorded SHA is the truth at the moment the event
  fired; correcting drift is a manual operation.

## Adoption checklist

Release Engineer:

- After T+5 escalation, label the maintainer task with the
  permission-gated-merge marker so the event source picks it up.
- Stop posting "still blocked" heartbeat comments. If the lane sits beyond
  T+60, NOD-347 silence applies; the merge event itself is the next signal.

Paperclip control plane (separate child issue):

- Subscribe the upstream repo's `pull_request` webhook to the canonical
  event router.
- Implement the handler against the contract above.
- Persist the idempotency key.
- Run `make release-merge-event-self-test` in CI as a regression gate.

QA Engineer:

- On wake from this contract, do not start verification until the
  upstream lane posts PostMerge_Proof. The wake is permission to prepare,
  not permission to execute.
