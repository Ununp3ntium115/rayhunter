# Rayhunter Orbic/shared issue and feature drain — Hermes + Claude collaborative prompt

Purpose

Work together through the complete Rayhunter contribution surface for `Ununp3ntium115/rayhunter`, implementing or documenting only work that is relevant to Orbic devices or to shared/general Rayhunter correctness, reliability, security, testing, build/release, documentation, or maintainability. The fork is the development/publishing surface; the upstream EFForg repository is the live issue evidence source because the fork has GitHub Issues disabled.

This prompt is the governing collaboration contract. The detailed 67-issue baseline queue, issue-to-card mapping, pseudocode, and acceptance baseline remain in:

`/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan/.hermes/context/rayhunter-exhaustive-loop-prompt.md`

Read that file completely before selecting any unit. It contains the exhaustive issue queue for upstream issues #1160 through #58, cards, scopes, pseudocode, additional branch lanes, missing #81/#78 mappings, source-maintenance markers, receipt fields, and stop conditions. Re-query every item live; the queue is not proof that an issue or branch is current.

Contribution and publication gate

Read `CONTRIBUTING.md` and `.hermes/context/rayhunter-ai-contribution-policy.md` before preparing any commit, PR, issue update, or review handoff. Do not post AI-generated issue/PR/review comments or discussion replies. Prepare private evidence only unless a human maintainer has reviewed and authorized the exact public text. Before any PR publication request, fill every applicable checklist item in the repository PR template. Public commits and pull requests must be authored by `ununp3ntum115` and nobody else; do not add AI/automation co-authors or trailers. Commit subjects must be human-approved, concise, imperative, and accurate to the actual diff; never claim tests or evidence that did not run.

Project identity and authority

- Repository: `/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan`
- Product: EFForg Rayhunter; never confuse this with Velociraptor Claw Edition.
- Fork remote: `https://github.com/Ununp3ntium115/rayhunter.git`
- Upstream evidence remote: `https://github.com/EFForg/rayhunter.git`
- Explicit Kanban board: `rayhunter-orbic`; invoke every board command with `HERMES_KANBAN_BOARD=rayhunter-orbic`.
- GitHub issues are disabled on the fork. Do not pretend fork issue queries are complete; query upstream issues, then map them to fork branches/cards.
- Do not mutate GitHub issues, PRs, labels, comments, branches, or releases without explicit operator authorization for the exact target.
- Do not publish AI-authored upstream issue text where the template requires claiming that no generative-AI tools were used. Prepare a human-reviewable draft and mark publication blocked.
- A board parent blocked intentionally is not permission to self-unblock. No card/file lease means inventory and disposition only.

Roles

- Hermes: orchestrator, live inventory, issue/card/branch reconciliation, scope control, evidence ledger, lease verification, independent readback, and next-unit selection.
- Claude Code: bounded investigator/implementer/reviewer in an isolated worktree, following one explicit directive and returning a durable receipt.
- Brody: authority for ambiguous scope, hardware/device actions, credentials, upstream publication, PR landing, issue mutation, and irreversible operations.

Live pickup gate — read-only first

```text
pickup():
  assert repository identity, fork remote, upstream remote, and board rayhunter-orbic
  read CLAUDE.md if present; if absent record MISSING_CONTEXT, never invent rules
  read every named .hermes/context file and the exhaustive queue
  git status --short --branch; git remote -v; git rev-parse HEAD; git log -5 --oneline
  query upstream open and closed issues; exclude pull requests from issue counts
  query fork open/closed PRs, branches, tags, and default branch
  query upstream PRs/linked commits for each candidate issue
  scan tracked source/docs/tests for TODO/FIXME/XXX/WIP/unfinished markers
  HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban stats
  HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban list --json
  inspect bridge channel tail, inbox pointers, newest outbox, active cursor, Claude PIDs/cwds
  reconcile issue -> card -> branch/commit -> current-tree proof, or record MISSING_MAPPING
  reconcile every dirty/untracked path to an owner or record UNKNOWN and HOLD
  write a fresh START receipt without overwriting prior evidence
```

If board identity, repository identity, ownership, or dirty-path attribution is contradictory, stop. Do not edit, build, test, scan, stage, commit, push, merge, clean, reset, stash, claim, unblock, or close anything.

Complete contribution surface to inventory

A. Upstream issue lanes

Process every issue in `rayhunter-exhaustive-loop-prompt.md`, currently the 67 selected direct-Orbic/shared-general baseline lanes, plus every newly surfaced live issue that passes the Orbic/shared-general scope filter. For every issue, capture live title/state/labels/author/updated time/full thread, related PRs/commits, likely paths/symbols, current-tree status, board card, branch, and one disposition:

`IMPLEMENT_NOW`, `NEEDS_OPERATOR_DECISION`, `NEEDS_HARDWARE_EVIDENCE`, `DUPLICATE_OR_ALREADY_FIXED`, `OUT_OF_SCOPE`, or `BLOCKED_EXTERNAL`.

Each issue section already provides issue-specific pseudocode. Replace generic assumptions with exact source symbols, callers, fixtures, commands, and evidence after live inspection. Never close an issue merely because a branch or commit exists.

### Supplemental live issues discovered after the baseline queue

The following upstream issues were open in the live re-query and are in scope for Orbic or shared/general work. They must not be silently omitted from the final inventory:

- #903 older mobile browsers cannot handle JS — shared frontend compatibility; inspect generated syntax/API support, reproduce on a constrained-browser fixture, and add the smallest compatibility or documented support-floor change.
- #895 unified, vendored shell server — shared installer/runtime architecture; inventory transports and privilege boundaries, define a common capability/error contract, and require disposable protocol fixtures.
- #793 ADB timeout after USB debug activation — shared ADB reliability candidate; trace readiness polling, add timeout/retry fixtures, and verify device scoping without live-device mutation.
- #542 installer-code deduplication — shared installer maintenance; identify duplicated seams, preserve device policy, refactor one bounded contract, and run cross-device installer tests.
- #586 ADB authentication research — shared security/reliability research; document threat model and device variants, then decide whether implementation is warranted before touching hardware.
- #334 adjustable thresholds/yellow line/periodic restart/warning display — shared configuration/UI feature; define backward-compatible semantics and require attended UI/UA proof for visual claims.
- #259 Orbic RC400L European LTE bands — direct Orbic hardware/regulatory feature; mock and document only until explicit operator and hardware authorization exists.
- #207 carrier unlock — direct Orbic/device and legal/operator gated; document feasibility and prerequisites, never perform a carrier or modem action autonomously.

Known live issues that are device-specific to TP-Link, Moxee, PinePhone, UZ801, Quectel, or TMOHS1 remain in the inventory as `OUT_OF_SCOPE` unless source inspection proves a shared/general regression. Record the issue number, reason, and evidence rather than dropping it silently.

B. Current fork PR lanes — inspect every one

At the last live inventory the fork had these open Aikido/security PRs:

- PR #8: enhanced-resolve 5.20.1 -> 5.24.4
- PR #7: Docker container default-root remediation
- PR #6: path-traversal remediation
- PR #5: path-traversal remediation
- PR #4: pin third-party GitHub Actions
- PR #3: dependency security updates across zerovec/open and other packages

It also had closed PR #2 (quick-xml security update) and closed PR #1 (Orbic serial response framing). Re-query the live PR list before action; these are not automatically trusted or merged.

For each PR:

```text
review_pr(pr):
  fetch PR metadata, diff, changed files, review comments, checks, merge state, and head/base SHAs
  verify the branch is in the fork and compare its base to current origin/main/upstream/main
  identify whether it maps to an upstream issue, a source TODO, a security finding, or duplicate work
  inspect every changed symbol and all callers; check path/device/security boundaries
  reproduce the claimed defect or prove the requested change with a deterministic fixture
  run the smallest relevant RED/GREEN or before/after check; capture full logs and exit codes
  audit dependencies/action pins/container privileges/path handling for regressions and supply-chain risk
  run repository quality gates required by the current tree
  classify LOCAL PASS/FAIL separately from provider CI and hardware evidence
  return APPROVE, REQUEST_CHANGES, HOLD_OPERATOR, DUPLICATE, or NOT_TESTED with evidence
```

Security PRs require especially strict review: an AI-generated or Aikido-created PR is not proof of correctness. Check lockfile consistency, transitive impact, reproducibility, exploit reachability, least privilege, path canonicalization, symlink/traversal behavior, and whether the fix creates a new compatibility or deployment regression. Do not merge or close without authorization.

C. Fork branches and existing feature work

Inventory all non-default fork branches and map each to an issue/card or mark it `MISSING_MAPPING`. At minimum inspect these known lanes when present: GPS browser location #1047, sparse analysis events #1139, GUI device connect #1137, 2G/3G PCAP #1013, recording display name #501, severity counts #363, timing advance #756, QMDL zip #58, Orbic Wi-Fi/client #1033/#589, Orbic charging/display #1160/#732, installer #510/#693/#901/#523, docs #1017/#1129/#591/#737/#769/#195/#833, and PCAP EOF/#730.

```text
reconcile_branch(branch):
  verify branch tip, base, author/source, ahead/behind, and changed-file list
  map branch to exact issue(s), card(s), and declared file lease
  compare branch to upstream/main and fork/main; inspect commit history and diff
  run branch-specific tests/gates in an isolated worktree
  check whether the branch is superseded, duplicated, partially fixed, unsafe, or ready for review
  never assume a branch is landable because it compiles
```

D. Source maintenance and untracked feature opportunities

Re-scan tracked source/docs/tests each cycle. Current known markers include:

- `.cargo/audit.toml`: rustls-rustcrypto exception follow-up
- `check/src/main.rs`: skip already-analyzed QMDL when matching PCAP
- `daemon/src/display/orbic.rs` and `tplink_framebuffer.rs`: display polling TODOs
- `installer/src/connection.rs`: expose command exit status
- `installer/src/uz801.rs`: variant-specific device-ID research
- `lib/src/analysis/connection_redirect_downgrade.rs` and `priority_2g_downgrade.rs`: SIB-state tracking
- `lib/src/analysis/information_element.rs`: NB message mapping FIXME
- `lib/src/pcap.rs`: radio-specific destination behavior
- `telcom-parser/README.md`: unfinished TODO section

Do not turn every TODO into a feature. For each marker, prove impact, assign a category and owner, create an idempotent card only when authorized, and keep maintenance work separate from issue implementation.

Shared implementation contract

```text
execute(unit):
  claim/reclaim only the explicitly named card at unit start
  declare issue/PR/branch, worktree, PID/session, exact owned files, exclusions, and lease
  read full issue/PR thread and current source; trace symbols, callers, sibling devices, and public boundaries
  define finite acceptance criteria and a deterministic RED/reproduction command
  capture complete RED output and nonzero exit when a defect is claimed
  add regression fixture/test first when practical
  implement the smallest root-cause change in leased files only
  run identical GREEN command and nearest caller/sibling tests
  for daemon web changes: npm install && npm run build -w daemon/web before Rust checks
  run cargo fmt --all --check, scoped cargo test/check/clippy, frontend checks, and git diff --check as applicable
  run secret/dependency/security review; never print or persist secrets
  run codegraph sync . after permitted source/docs changes when available
  obtain independent correctness and production-safety review
  distinguish LOCAL, PROVIDER_CI, HARDWARE, and UI/UA evidence
  write and read back a durable receipt; do not self-select a follow-on
```

Orbic safety rules

Never identify Orbic from Qualcomm USB vendor `0x05c6` alone. Require vendor + product + interface + serial/profile constraints. Never run installer, ADB, serial, Wi-Fi, firmware, rootshell, or device-control operations against connected hardware without explicit operator authorization and a disposable/safe target. Preserve separate device-specific workarounds; do not globalize a fix based on one device. UI/UA/CUA claims require attended evidence and operator PASS/FAIL, not source tests.

Receipt contract

```yaml
unit: <issue, PR, branch, or maintenance ID>
classification: IMPLEMENT_NOW|NEEDS_OPERATOR_DECISION|NEEDS_HARDWARE_EVIDENCE|DUPLICATE_OR_ALREADY_FIXED|OUT_OF_SCOPE|BLOCKED_EXTERNAL|IN_PROGRESS|VERIFIED_LOCAL|LANDED_UNVERIFIED_PROVIDER
repository: /Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan
board: rayhunter-orbic
card: <id or MISSING_MAPPING>
issue_or_pr: <number/url or none>
worktree: <absolute path or null>
branch: <name or null>
owned_files: []
commands: [{command: <exact>, exit_code: <int>, log: <absolute path>}]
local_tests: PASS|FAIL|BLOCKED|NOT_TESTED
provider_ci: PASS|FAIL|PENDING|BLOCKED|NOT_RUN
hardware: PASS|FAIL|BLOCKED|NOT_TESTED|NOT_APPLICABLE
ui_ua: PASS|FAIL|BLOCKED|NOT_TESTED|NOT_APPLICABLE
review: APPROVE|REQUEST_CHANGES|PENDING|NOT_TESTED
commit: <sha or none>
staged_files: []
remaining_dirty_paths: []
blockers: []
operator_decision_needed: []
next_action: <one bounded action>
publication_status: NOT_REQUESTED|DRAFT_ONLY|AUTHORIZED|BLOCKED
release_readiness: NOT_CLAIMED
```

Batch/heartbeat rules

- Hermes proposes the next 3–5 highest-value units; Claude executes at most one mutation unit at a time.
- One build/test owner and one disjoint file lease at a time.
- Every directive has a unique ACK token, Markdown packet, sibling pointer YAML, exact outbox path, and stop conditions.
- `/hb` is read-only. If no material state changed, emit exactly `[SILENT]`; if state changed, report only verified deltas and one next action.
- A process, branch, commit, card count, filtered test, or chat claim is not completion proof.
- Stop on overlap, unexplained dirty path, missing lease, board/repository mismatch, missing RED/GREEN, hardware gap, credential/operator gate, unexpected publication, or contradictory evidence.
- Full drain completion requires every issue, PR, branch, TODO/maintenance lane, and missing mapping to have a current evidence-backed disposition. Never claim release readiness from inventory completion.

Start now

Perform read-only pickup, refresh upstream issues and fork PRs/branches, scan maintenance markers, reconcile them to `rayhunter-orbic`, and produce the first evidence-backed inventory/disposition receipt. Do not mutate code, GitHub, or the board until an explicit bounded unit and file lease are authorized.
