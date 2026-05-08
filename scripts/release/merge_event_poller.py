#!/usr/bin/env python3
"""Reference handler for the merge-event automation contract.

Implements the post-merge transition fan-out described in
docs/release/merge-event-automation.md as a polling-shim. Until the
control-plane webhook subscription lands, this script can run on a routine
to drive event-driven wakeups; once the webhook lands, the handler logic
here is the canonical reference for the control-plane implementation.

The script is intentionally side-effect-light by default:
    --self-test          : run the replay-safety demo and exit non-zero on failure
    --event-json X --links-json Y
                         : dry-run plan from on-disk fixture inputs
    --from-paperclip ID  : pull live linked-issue state from the Paperclip API
                           and live PR state from GitHub, then dry-run plan
    --apply              : not implemented; this is the contract reference

Idempotency is the contract:
    key = f"merge-event:{repo}:{number}:{merge_commit_sha}"
The handler refuses to re-apply a stamped key. The self-test asserts this.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.request
from dataclasses import dataclass, field
from typing import Iterable


@dataclass(frozen=True)
class MergeEvent:
    repo: str
    number: int
    url: str
    head_sha: str
    merge_commit_sha: str
    merged_at: str
    merged_by: str

    @property
    def idempotency_key(self) -> str:
        return f"merge-event:{self.repo}:{self.number}:{self.merge_commit_sha}"


@dataclass(frozen=True)
class LinkedIssue:
    identifier: str
    role: str  # "maintainer_task" | "merge_blocked" | "qa_downstream" | "mention"
    status: str
    assignee: str | None


@dataclass
class PlannedTransition:
    issue: str
    role: str
    status_from: str
    status_to: str | None  # None = no status change
    comment: str
    wake: str | None  # assignee role to wake


@dataclass
class HandlerResult:
    transitions: list[PlannedTransition] = field(default_factory=list)
    skipped_reason: str | None = None

    def is_noop(self) -> bool:
        return not self.transitions and self.skipped_reason is not None


# In-memory dedupe store. The control-plane impl substitutes a persistent
# store (issue label, dedupe table, evidence-comment lookup, etc).
_DEDUPE_STORE: set[str] = set()


def _evidence_comment(event: MergeEvent, role: str) -> str:
    body = (
        "## Merged\n\n"
        f"PR: {event.url}\n"
        f"Merged at: {event.merged_at}\n"
        f"Merged by: {event.merged_by}\n"
        f"Merge SHA: `{event.merge_commit_sha}`\n\n"
        "Triggered automatically by merge-event automation "
        "(docs/release/merge-event-automation.md).\n\n"
        f"Idempotency key: `{event.idempotency_key}`"
    )
    if role == "merge_blocked":
        body += (
            "\n\nNext: Release Engineer publishes PostMerge_Proof "
            "(deploy + smoke). QA lane remains gated on that proof per "
            "docs/release/permission-gated-merge-escalation.md."
        )
    elif role == "qa_downstream":
        body += (
            "\n\nWait for PostMerge_Proof on the upstream lane before "
            "starting verification."
        )
    return body


def _plan_one(event: MergeEvent, issue: LinkedIssue) -> PlannedTransition:
    if issue.role == "maintainer_task":
        target = "done"
        wake = None
    elif issue.role == "merge_blocked":
        target = "in_progress"
        wake = "release"
    elif issue.role == "qa_downstream":
        target = "in_progress"
        wake = "qa"
    else:
        target = None
        wake = None

    return PlannedTransition(
        issue=issue.identifier,
        role=issue.role,
        status_from=issue.status,
        status_to=target,
        comment=_evidence_comment(event, issue.role),
        wake=wake,
    )


def handle(
    event: MergeEvent,
    linked: Iterable[LinkedIssue],
    *,
    dedupe_store: set[str] | None = None,
) -> HandlerResult:
    """Plan transitions for one merge event.

    Idempotency: if the event's key is already in the dedupe store, returns
    a no-op result. Otherwise, returns the full plan and stamps the key.
    """
    store = dedupe_store if dedupe_store is not None else _DEDUPE_STORE

    if event.idempotency_key in store:
        return HandlerResult(
            transitions=[],
            skipped_reason=f"already applied: {event.idempotency_key}",
        )

    transitions = [_plan_one(event, i) for i in linked]
    store.add(event.idempotency_key)
    return HandlerResult(transitions=transitions)


def _self_test() -> int:
    fixture_event = MergeEvent(
        repo="kcirtapfromspace/no_drake_in_the_house",
        number=5402,
        url="https://github.com/kcirtapfromspace/no_drake_in_the_house/pull/5402",
        head_sha="deadbeefcafedeadbeefcafedeadbeefcafedead",
        merge_commit_sha="0abc8d29201eaefe0dfed8b106e619e586047489",
        merged_at="2026-05-07T22:17:48Z",
        merged_by="kcirtapfromspace",
    )
    fixture_links = [
        LinkedIssue("NOD-378", "maintainer_task", "blocked", "release"),
        LinkedIssue("NOD-363", "merge_blocked", "blocked", "release"),
        LinkedIssue("NOD-366", "qa_downstream", "blocked", "qa"),
    ]

    store: set[str] = set()
    first = handle(fixture_event, fixture_links, dedupe_store=store)
    second = handle(fixture_event, fixture_links, dedupe_store=store)

    failures: list[str] = []
    if len(first.transitions) != 3:
        failures.append(
            f"first run: expected 3 transitions, got {len(first.transitions)}"
        )
    if first.skipped_reason is not None:
        failures.append(
            f"first run: expected no skip, got {first.skipped_reason!r}"
        )
    if not second.is_noop():
        failures.append(
            f"second run: expected no-op, got {len(second.transitions)} "
            f"transitions, skip={second.skipped_reason!r}"
        )

    expected_targets = {
        "NOD-378": "done",
        "NOD-363": "in_progress",
        "NOD-366": "in_progress",
    }
    for t in first.transitions:
        if expected_targets.get(t.issue) != t.status_to:
            failures.append(
                f"{t.issue}: expected status_to={expected_targets.get(t.issue)!r}, "
                f"got {t.status_to!r}"
            )

    expected_wakes = {"NOD-378": None, "NOD-363": "release", "NOD-366": "qa"}
    for t in first.transitions:
        if expected_wakes.get(t.issue) != t.wake:
            failures.append(
                f"{t.issue}: expected wake={expected_wakes.get(t.issue)!r}, "
                f"got {t.wake!r}"
            )

    if failures:
        print("SELF-TEST FAILED:", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1

    print("SELF-TEST PASSED")
    print(f"  first run:  {len(first.transitions)} transitions planned")
    print(
        f"  second run: 0 transitions (skipped: "
        f"{second.skipped_reason})"
    )
    return 0


# --- Live state collection ----------------------------------------------------
#
# These helpers are intentionally read-only. The shim never writes to either
# Paperclip or GitHub; it only assembles the inputs that handle() consumes.

_PR_URL_RE = re.compile(
    r"https?://github\.com/(?P<owner>[^/]+)/(?P<repo>[^/]+)/pull/(?P<number>\d+)"
)


def _paperclip_get(path: str) -> dict:
    base = os.environ["PAPERCLIP_API_URL"].rstrip("/")
    token = os.environ["PAPERCLIP_API_KEY"]
    req = urllib.request.Request(
        f"{base}{path}",
        headers={"Authorization": f"Bearer {token}"},
    )
    with urllib.request.urlopen(req) as resp:
        return json.load(resp)


def _paperclip_find_issue(identifier: str) -> dict:
    company = os.environ["PAPERCLIP_COMPANY_ID"]
    listing = _paperclip_get(
        f"/api/companies/{company}/issues?q={identifier}"
    )
    rows = listing if isinstance(listing, list) else listing.get("issues") or []
    match = next((r for r in rows if r.get("identifier") == identifier), None)
    if not match:
        raise SystemExit(f"no issue found for identifier {identifier!r}")
    return _paperclip_get(f"/api/issues/{match['id']}")


def _gh_pr_view(owner: str, repo: str, number: int) -> dict:
    out = subprocess.check_output(
        [
            "gh",
            "pr",
            "view",
            str(number),
            "--repo",
            f"{owner}/{repo}",
            "--json",
            "state,mergedAt,mergedBy,mergeCommit,headRefOid,url",
        ],
        text=True,
    )
    return json.loads(out)


def _build_event_from_live(maintainer_issue: dict) -> MergeEvent | None:
    body = maintainer_issue.get("description") or ""
    m = _PR_URL_RE.search(body)
    if not m:
        raise SystemExit(
            "no PR URL found in maintainer-task description; cannot resolve event"
        )
    owner, repo, number = m["owner"], m["repo"], int(m["number"])
    pr = _gh_pr_view(owner, repo, number)
    if pr.get("state") != "MERGED":
        return None
    merge_commit = (pr.get("mergeCommit") or {}).get("oid") or ""
    merged_by = (pr.get("mergedBy") or {}).get("login") or "unknown"
    return MergeEvent(
        repo=f"{owner}/{repo}",
        number=number,
        url=pr.get("url") or f"https://github.com/{owner}/{repo}/pull/{number}",
        head_sha=pr.get("headRefOid") or "",
        merge_commit_sha=merge_commit,
        merged_at=pr.get("mergedAt") or "",
        merged_by=merged_by,
    )


def _build_links_from_live(maintainer_issue: dict) -> list[LinkedIssue]:
    """Walk the relatedWork graph rooted at the maintainer task.

    Roles are inferred by graph position:
      - the maintainer task itself = "maintainer_task"
      - each outbound mention = "merge_blocked"
      - each issue any merge-blocked lane `blocks` (one hop further) = "qa_downstream"
    """
    links: list[LinkedIssue] = []
    links.append(
        LinkedIssue(
            identifier=maintainer_issue["identifier"],
            role="maintainer_task",
            status=maintainer_issue.get("status") or "unknown",
            assignee=maintainer_issue.get("assigneeAgentId"),
        )
    )

    related = maintainer_issue.get("relatedWork") or {}
    seen: set[str] = {maintainer_issue["identifier"]}
    for entry in related.get("outbound") or []:
        issue = entry.get("issue") or {}
        ident = issue.get("identifier")
        if not ident or ident in seen:
            continue
        # Skip references to policy/parent docs that are clearly not blockers.
        if (issue.get("status") or "").lower() == "done":
            continue
        seen.add(ident)
        links.append(
            LinkedIssue(
                identifier=ident,
                role="merge_blocked",
                status=issue.get("status") or "unknown",
                assignee=issue.get("assigneeAgentId"),
            )
        )
        # Hop one further: anything this merge-blocked lane `blocks` is QA-downstream.
        try:
            full = _paperclip_get(f"/api/issues/{issue['id']}")
        except Exception:  # pragma: no cover - best-effort
            continue
        for downstream in full.get("blocks") or []:
            d_ident = downstream.get("identifier")
            if not d_ident or d_ident in seen:
                continue
            seen.add(d_ident)
            links.append(
                LinkedIssue(
                    identifier=d_ident,
                    role="qa_downstream",
                    status=downstream.get("status") or "unknown",
                    assignee=downstream.get("assigneeAgentId"),
                )
            )
    return links


def _from_paperclip(identifier: str) -> int:
    issue = _paperclip_find_issue(identifier)
    event = _build_event_from_live(issue)
    links = _build_links_from_live(issue)

    print(f"# Live dry-run for {identifier} ({issue.get('title')})")
    print()
    print("## Linked issues (graph walk)")
    for link in links:
        print(f"  - {link.identifier} [{link.role}] status={link.status}")
    print()

    if event is None:
        print("## PR state: not merged yet")
        print(
            "  No merge event to fire. The shim would re-poll on the next cycle."
        )
        return 0

    print("## Merge event")
    print(f"  url: {event.url}")
    print(f"  merge_commit_sha: {event.merge_commit_sha}")
    print(f"  merged_at: {event.merged_at}")
    print(f"  merged_by: {event.merged_by}")
    print(f"  idempotency_key: {event.idempotency_key}")
    print()

    print("## Planned transitions (dry-run)")
    result = handle(event, links, dedupe_store=set())
    print(_format_plan(result))
    return 0


def _format_plan(result: HandlerResult) -> str:
    if result.is_noop():
        return f"no-op ({result.skipped_reason})"
    lines = []
    for t in result.transitions:
        change = (
            f"{t.status_from} -> {t.status_to}" if t.status_to else "no status change"
        )
        wake = f", wake={t.wake}" if t.wake else ""
        lines.append(f"{t.issue} [{t.role}]: {change}{wake}")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--self-test", action="store_true", help="run replay-safety demo")
    p.add_argument("--dry-run", action="store_true", default=True)
    p.add_argument("--apply", action="store_true", help="not implemented in shim")
    p.add_argument(
        "--event-json",
        type=str,
        help="path to a merge event JSON file (see docs for schema)",
    )
    p.add_argument(
        "--links-json",
        type=str,
        help="path to a linked-issues JSON file: list of {identifier, role, status, assignee}",
    )
    p.add_argument(
        "--from-paperclip",
        type=str,
        metavar="IDENTIFIER",
        help=(
            "pull live linked-issue state from Paperclip + live PR state from "
            "GitHub for the given maintainer-task identifier (e.g. NOD-378), "
            "then dry-run the plan"
        ),
    )
    args = p.parse_args(argv)

    if args.self_test:
        return _self_test()

    if args.apply:
        print(
            "--apply is not implemented in the shim; this is the contract "
            "reference. Wire the control-plane handler against handle().",
            file=sys.stderr,
        )
        return 2

    if args.from_paperclip:
        return _from_paperclip(args.from_paperclip)

    if not args.event_json or not args.links_json:
        print(
            "usage: --self-test, OR --from-paperclip IDENTIFIER, "
            "OR --event-json X --links-json Y for a dry-run plan",
            file=sys.stderr,
        )
        return 2

    with open(args.event_json) as f:
        event = MergeEvent(**json.load(f))
    with open(args.links_json) as f:
        links = [LinkedIssue(**row) for row in json.load(f)]
    result = handle(event, links)
    print(_format_plan(result))
    return 0


if __name__ == "__main__":
    sys.exit(main())
