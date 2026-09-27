# Rayhunter ranked SDLC pipeline: TODO and pseudocode generation prompt

Use this prompt from the Rayhunter checkout only. The purpose is to convert the ranked issue/PR list below into durable Kanban TODOs, implementation-ready pseudocode, bounded work lanes, and verified commits without losing branch or worktree discipline.

## Mandatory repository and branch guard

Before creating cards, editing files, staging, committing, or pushing:

1. Confirm the repository is `/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan`.
2. Confirm the repository is Rayhunter, with:
   - `origin` pointing to `https://github.com/Ununp3ntium115/rayhunter.git`
   - `upstream` pointing to `https://github.com/EFForg/rayhunter.git`
3. Confirm the current branch is exactly `feature/sdlc-drain-v1`.
4. Confirm the current worktree is the intended tree and not `/Users/brodynielsen/GitRepos/Orca/.worktrees/t_3f4ea929` or any unrelated checkout.
5. Record `git status --short --branch`, `git log -1`, and all pre-existing dirty/untracked paths. Preserve those paths. Do not reset, clean, stash, checkout another branch, or overwrite unrelated changes.
6. If any guard fails, stop with `BLOCKED_WRONG_TREE_OR_BRANCH`; do not create implementation commits.
7. Use one bounded mutation lane and one file lease at a time. Each issue or PR must have its own card and explicit scope.

The canonical integration branch is `feature/sdlc-drain-v1`. Every implementation commit for this pipeline must be made on that branch, must contain only the authorized lane's files, and must be verified with `git show --stat`, `git status`, and the relevant tests. Do not commit to `main`, `master`, `develop`, `integration`, or an issue/PR branch unless the operator explicitly changes this instruction.

Public commits and PRs must be authored by `Ununp3ntium115` only. Do not add AI, Hermes, Claude, automation, or co-author trailers. Do not publish GitHub comments, issue text, PR descriptions, or review comments without exact human approval. Keep evidence and drafts private.

## Operating objective

Drain the ranked Rayhunter backlog into a complete SDLC pipeline:

- create or reconcile one Kanban TODO per issue/feature;
- include issue-specific pseudocode, affected paths, acceptance criteria, RED/GREEN tests, security review, hardware-evidence requirements, and publication constraints;
- prioritize blockers and safety-sensitive installer/device work before polish;
- implement only one leased lane at a time;
- verify every result against live Git state and command output;
- commit only verified work to `feature/sdlc-drain-v1`;
- never claim hardware validation, provider CI, upstream publication, or issue closure without evidence.

Use explicit board slug `rayhunter-orbic` and the worktree above. The parent drain card is `t_bf6d3c0a`; do not self-unblock parent-gated children. If the parent is still blocked, create/reconcile detailed child cards and report the gate rather than bypassing it.

## Ranked work inventory

Already completed or previously addressed; reconcile against live Git and issue state before creating duplicate work:

- #1129 — RC400L 5G documentation fix
- #1160 — Orbic charging display bug
- #1033 — Wi-Fi client activation fix

Quick wins:

- #967 — three password/device/firmware bugs
- #1144 — extract shared UI components

Medium effort:

- #1047 — browser geolocation API / GPS feature
- #1133 — persist device metadata for recordings
- #1157 — TP-Link M7200 v4.0 SSL error
- #981 — PinePhone installation issues
- #978 — Moxee hotspot installation issues
- #1105 — TP-Link M7350 installation issues

Larger or open PR work:

- #1142 — LTE LPP ASN.1 bindings; Phil Gebhardt / telcom-parser work
- #1042 — Wi-Fi-based heuristics; Jack Lund PR / complex feature
- #1154 — package-lock.json regeneration / maintenance

Recommended execution order unless live evidence changes the priority:

1. #967 — high-value quick bugs
2. #1047 — browser geolocation
3. #1133 — recording metadata persistence
4. #1144 — shared UI extraction
5. #1157, #981, #978, #1105 — device/installer lanes, ordered by blocker severity and available hardware evidence
6. #1154 — lockfile maintenance
7. #1142 — LPP bindings
8. #1042 — Wi-Fi heuristics

The order is a recommendation, not permission to bypass board gates or operator decisions.

## Required TODO/card format

For every candidate, create or reconcile a card with this structure. If a card already exists, update only through the authorized Kanban workflow and preserve its history.

Title:
`Rayhunter #<number>: <short action> — TODO / pseudocode / verification`

Body sections:

1. Issue identity and live URL.
2. Classification: `direct-orbic`, `shared-general`, `installer`, `frontend`, `daemon`, `analysis`, `docs`, `test`, `security`, `build`, or `maintenance`.
3. Disposition: `IMPLEMENT_NOW`, `NEEDS_OPERATOR_DECISION`, `NEEDS_HARDWARE_EVIDENCE`, `DUPLICATE_OR_ALREADY_FIXED`, `OUT_OF_SCOPE`, or `BLOCKED_EXTERNAL`.
4. Current live status and duplicate/already-fixed check.
5. Exact worktree and branch guard.
6. Scope boundaries and files/symbols to inspect.
7. Issue-specific pseudocode.
8. RED tests to add or run before implementation.
9. GREEN implementation and tests.
10. Acceptance criteria, including failure behavior.
11. Security and compatibility review.
12. Hardware matrix and required screenshots/logs/PCAPs where applicable.
13. Commit scope and proposed commit subject.
14. Publication policy and human-review gate.
15. Blockers and operator decisions.

## Pseudocode templates by candidate

### #967 — password/device/firmware bugs

```text
load the complete issue and split the three reported defects into three independently testable subcases
for each subcase:
    identify the user-visible failure and the exact device/firmware conditions
    trace frontend -> API/daemon -> device/config persistence call chain
    reproduce with a deterministic fixture or mocked device response
    write a RED regression test for the failing behavior
    implement the narrowest fix without weakening authentication or device targeting
    verify malformed, missing, stale, and unauthorized inputs fail safely
    run targeted tests plus formatting/lint/build checks
    record whether real hardware and firmware versions were tested
keep unrelated subcases in separate commits unless the repository's existing change naturally requires one atomic fix
```

### #1144 — shared UI components

```text
inventory repeated UI markup, state handling, accessibility labels, and error/loading patterns
select one component boundary at a time; do not redesign behavior while extracting
write characterization tests or snapshots for current behavior
extract a typed shared component with explicit props and stable defaults
replace callers one at a time
verify keyboard/accessibility behavior, loading/error states, and responsive layout
run frontend build and relevant tests; compare generated output for unintended changes
```

### #1047 — browser geolocation API

```text
confirm browser secure-context and permission requirements
require explicit user opt-in before requesting location
check permission state and handle denied, unavailable, timeout, and malformed coordinates
validate latitude/longitude ranges and timestamp freshness
send location only through an explicit authenticated endpoint with origin/CORS policy constrained by configuration
never log exact coordinates or expose them in public evidence
write unit tests for permission/error/validation paths and integration tests for the configured endpoint
verify behavior on localhost/HTTPS and document browser limitations
```

### #1133 — recording device metadata persistence

```text
identify recording creation, rotation, serialization, database, export, and replay paths
define a versioned metadata schema with nullable fields for backward compatibility
capture device model, firmware, capture mode, and safe identifiers at recording start
persist atomically with the recording; do not lose capture data if metadata is unavailable
migrate/read old recordings without inventing values
exclude secrets, subscriber identifiers, and sensitive location from default exports/logs
write RED tests for absent metadata, restart/recovery, old schema, corrupt metadata, and concurrent recording creation
run storage, serialization, export, and regression tests
```

### #1157 — TP-Link M7200 v4.0 SSL error

```text
identify the exact firmware/version and TLS failure from sanitized logs
reproduce with a fixture or controlled device test; do not disable certificate validation globally
inspect protocol negotiation, hostname/SNI, CA bundle, clock, and device endpoint behavior
apply a device-scoped compatibility path only when positively identified
retain secure defaults and fail closed for unknown devices
add regression tests for the known firmware and for rejection of unrelated devices
require hardware evidence or an operator-approved sanitized trace before claiming fixed
```

### #981 / #978 / #1105 — PinePhone, Moxee, and TP-Link M7350 installers

```text
capture the exact OS/device/firmware/install step and failure output
trace installer preflight, dependency checks, USB/ADB/serial/network detection, and rollback behavior
add a deterministic preflight fixture for the failure
write RED tests for wrong device, missing dependency, timeout, partial install, and retry
implement a device-scoped fix with explicit identity checks; never match a broad vendor ID alone
ensure failed installs leave a recoverable state and do not destroy user data
run installer unit/integration tests and, when available, controlled hardware validation
record screenshots, logs, firmware, and operator pass/fail separately
```

### #1154 — package-lock regeneration

```text
read package.json workspaces and lockfile policy
regenerate with the repository-approved Node/npm version and no unrelated package upgrades
inspect the full lockfile diff for registry/source/integrity anomalies
run npm install --package-lock-only or the documented equivalent
run frontend/daemon build and dependency audit gates
commit only package-lock changes and verified generated output
```

### #1142 — LTE LPP ASN.1 bindings

```text
read the issue, linked PR, telcom-parser schema, generated-code policy, and current parser callers
identify whether generated bindings are missing, stale, or incompatible
create fixture-based decode/encode tests from sanitized protocol samples
regenerate using the pinned toolchain, if required; do not hand-edit generated output
verify unknown extensions and malformed messages are handled safely
run parser tests, Rust formatting, Clippy, and downstream analysis tests
separate generated-code changes from hand-written integration changes where practical
```

### #1042 — Wi-Fi-based heuristics

```text
read the complete PR and compare its base with upstream/main and origin/main
trace packet/event inputs, feature extraction, thresholds, scoring, and alert emission
define false-positive/false-negative expectations and device capability limits
write RED fixtures for empty, noisy, duplicate, malformed, and normal Wi-Fi observations
implement behind an explicit configuration/feature gate if behavior is experimental
verify no subscriber identifiers or raw sensitive payloads enter logs or public evidence
run analysis tests, benchmark/resource checks, and cross-device regression tests
require maintainer review before merge or publication
```

## Universal verification and commit gate

Before marking a lane ready:

- re-run `git status --short --branch` and confirm branch is exactly `feature/sdlc-drain-v1`;
- confirm only the card's authorized files are changed/staged;
- run `git diff --check`;
- run the narrowest relevant tests first, then repository-required format/build/lint gates;
- distinguish local pass, provider CI pending/pass/fail, and hardware evidence;
- run `codegraph sync .` after code or documentation changes when available;
- inspect `git diff --cached --stat` and `git diff --cached --check`;
- commit only on `feature/sdlc-drain-v1` with a truthful subject matching the diff;
- verify with `git show --stat --oneline HEAD`, `git status --short --branch`, and (when authorized) `git log origin/feature/sdlc-drain-v1..HEAD`;
- never push, open/close/merge PRs, comment publicly, or claim release readiness without explicit human authorization.

Suggested commit format:

`fix: <issue-specific behavior>`
`feat: <issue-specific capability>`
`docs: <documentation-only change>`
`test: <regression coverage>`
`chore: regenerate package lockfile`

Do not use vague subjects such as `AI fixes`, `automated work`, or `misc updates`.

## Required pipeline output

Return a durable, machine-readable receipt for every cycle:

```yaml
repository: /Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan
branch: feature/sdlc-drain-v1
board: rayhunter-orbic
parent_gate: t_bf6d3c0a
cards_created_or_reconciled: []
selected_lane: null
preexisting_dirty_paths: []
changed_paths: []
commands_run: []
tests_passed: []
tests_failed: []
hardware_evidence: not-run
provider_ci: not-run
commit: null
push: not-authorized
public_mutation: not-authorized
blockers: []
next_action: null
```

If there is no valid lease or the parent gate remains blocked, create/reconcile TODOs and output the proposed next 3–5 lanes without mutating source. If no meaningful state changed, report `NO_MATERIAL_CHANGE` and stop.
