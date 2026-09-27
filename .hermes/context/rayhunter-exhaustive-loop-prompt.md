# Rayhunter `/loop` master execution directive

**Purpose:** drain the complete Rayhunter Orbic/shared-feature issue program without inventing status, mixing work, or claiming hardware/CI evidence that was not produced. This file is the canonical worker prompt loaded by `.claude/commands/loop.md`.

## 0. Identity and live-state gate

- Repository: `/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan`
- Product: EFForg `rayhunter`; this is not Velociraptor Claw Edition.
- Hermes board: `rayhunter-orbic`; every Kanban command must set `HERMES_KANBAN_BOARD=rayhunter-orbic`.
- Upstream evidence source: `EFForg/rayhunter`; the fork has Issues disabled. Never mutate GitHub issues, labels, comments, branches, or PRs without explicit operator authorization.
- Current observations are stale until re-read. At generation time the checkout was observed on `doc/adding-heuristic-1017`, `package-lock.json` was modified, coordination directories were untracked, the board showed 71 TODO and 1 blocked parent, and no `CLAUDE.md` was present in the checkout. Preserve all of that unless a later live read proves ownership and authorizes a bounded change.
- Existing Claude bridge history contains Claw/Rayhunter cross-project HOLD messages and a Rayhunter ACK saying no READY card was available. Treat those as historical evidence, not permission.

## 1. Mission and completion semantics

At every invocation, refresh the live GitHub issue inventory, local Git state, board state, bridge state, and active process/worktree ownership. Reconcile the inventory against the 71 existing child cards, the 3 branch-review lanes, the source maintenance lane, and any newly discovered open issue. Process one bounded unit at a time. The full program is complete only when every in-scope issue/lane has an evidence-backed disposition and every actionable unit has an independently verified receipt.

Allowed dispositions: `IMPLEMENT_NOW`, `NEEDS_OPERATOR_DECISION`, `NEEDS_HARDWARE_EVIDENCE`, `DUPLICATE_OR_ALREADY_FIXED`, `OUT_OF_SCOPE`, `BLOCKED_EXTERNAL`, `IN_PROGRESS`, `VERIFIED_LOCAL`, `LANDED_UNVERIFIED_PROVIDER`, `DONE_WITH_EVIDENCE`. Never equate a card count, commit, filtered test, process, or worker message with completion.

Priority: (1) wrong-device/data-loss/installer/connectivity/safety risks; (2) shared correctness affecting Orbic and other devices; (3) security/build/test/release blockers; (4) high-value general features; (5) docs and polish. UI/UA/CUA work remains a separate attended lane and cannot be silently closed from source tests.

## 2. Pickup protocol (mandatory before any mutation)

```text
pickup():
  assert git rev-parse --show-toplevel == repo
  read CLAUDE.md if present; if absent record MISSING_CONTEXT and do not invent its rules
  read .hermes/context/rayhunter-orbic-review-20260925-session-01a0da5b-context.md if present
  read .hermes/context/rayhunter-backlog-map-20260925.md
  git status --short --branch; git remote -v; git log -3 --oneline
  HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban stats
  HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban list --json > evidence/loop/<run>/board.json
  inspect bridge channel tail, inbox pointers, outbox receipts, active cursor, Claude PIDs/cwds
  query live upstream issues and exclude pull requests
  reconcile every issue to a card, branch, current-tree proof, or explicit missing-card record
  assert no overlapping lease and no unexplained dirty path
  write START receipt with UTC, HEAD, counts, hashes, owners, and stop conditions
```

If the board parent is blocked or no explicit card/file lease exists, do not self-unblock, self-select, edit, build, test, stage, commit, or push. Produce the inventory/disposition packet and wait for operator selection. Do not use parent links for governance because they freeze children.

## 3. Per-unit implementation contract

```text
execute(unit):
  re-read issue, linked discussion, current source, callers, sibling-device paths, and card
  classify direct-orbic/shared-general/docs/tests/security/release/maintenance
  declare exact worktree, branch, card, owner, PID/session, and exclusive file lease
  write acceptance checklist and deterministic RED command/log
  reproduce or prove the reported behavior; do not fabricate reproduction
  add regression fixture/test first where practical
  implement smallest root-cause change; preserve unrelated dirty paths
  run focused GREEN test and nearest caller/sibling tests
  if daemon web is involved: npm install && npm run build -w daemon/web before Rust checks
  run cargo fmt --all --check, relevant cargo test/check/clippy, frontend checks, git diff --check
  run scoped secret/dependency/security review and codegraph sync . after permitted edits
  separate LOCAL, PROVIDER_CI, and HARDWARE evidence
  have an independent review read exact changed files and tests
  write/read-back receipt; stage only explicitly owned files for Claude commit handoff
  never publish/merge/close/label upstream without operator authorization
```

Safety rules: Qualcomm USB vendor `0x05c6` alone never identifies Orbic; device-control requires vendor+product+interface+serial/profile constraints. Never type or persist secrets. Never touch the operator's live host or connected hardware without explicit authorization. AI-generated work must be understood, tested, and disclosed by the maintainer.

## 4. Required receipt

```yaml
run_id: <stable timestamp/id>
unit: <issue/card/branch/maintenance id>
classification: <one allowed disposition>
worktree: <absolute path or null>
branch: <branch or null>
owned_files: []
commands: [{command: <text>, exit_code: <int>, log: <path>}]
local_tests: {status: PASS|FAIL|BLOCKED|NOT_TESTED, scope: TARGETED|FULL, evidence: []}
provider_ci: PASS|FAIL|PENDING|BLOCKED|NOT_RUN
hardware: PASS|FAIL|BLOCKED|NOT_TESTED|NOT_APPLICABLE
security: PASS|FAIL|BLOCKED|NOT_TESTED
review: APPROVE|REJECT|PENDING
commit: <sha or none>
staged_files: []
remaining_dirty_paths: []
blockers: []
next_action: <one bounded action>
publication_status: NOT_REQUESTED|DRAFT_ONLY|AUTHORIZED|BLOCKED
```

## 5. Exhaustive issue/feature work queue

The following queue was generated from the prior inventory. Re-query each issue before action; titles and card mappings are not proof of current state. For every line apply the listed pseudocode, then add exact source paths, tests, and evidence to the receipt.

### #1160 — Full screen status incorrect while off but charging on orbic
- Scope: `direct-orbic`
- Board card: `t_e7edf1c3`
- Evidence: https://github.com/EFForg/rayhunter/issues/1160
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1144 — Extract shared components for GUI
- Scope: `shared-general`
- Board card: `t_4ec82505`
- Evidence: https://github.com/EFForg/rayhunter/issues/1144
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1139 — Refactor analysis format to permit more than one event per (analyzer, msg)
- Scope: `shared-general`
- Board card: `t_f0f1b9e7`
- Evidence: https://github.com/EFForg/rayhunter/issues/1139
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1137 — add a device connection screen to the GUI installer
- Scope: `direct-orbic`
- Board card: `t_d00b5a12`
- Evidence: https://github.com/EFForg/rayhunter/issues/1137
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1133 — Persist device metadata for recordings
- Scope: `shared-general`
- Board card: `t_78ae8d77`
- Evidence: https://github.com/EFForg/rayhunter/issues/1133
- Pseudocode: `trace_recording_lifecycle(); define_limits_and_failure_semantics(); add_persistence_and_corruption_tests(); implement_atomic_bounded_operation(); verify_redaction_and_recovery();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1129 — RC400L documentation mentions it supports 5G bands when it is not a 5G device
- Scope: `direct-orbic`
- Board card: `t_74c07a6e`
- Evidence: https://github.com/EFForg/rayhunter/issues/1129
- Pseudocode: `read_current_docs_and_source_of_truth(); identify_stale_or_missing_steps(); write_minimal_canonical_section(); validate_commands_and_links(); run_diff_check();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1072 — LPP heuristic
- Scope: `shared-general`
- Board card: `t_c8163937`
- Evidence: https://github.com/EFForg/rayhunter/issues/1072
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1047 — Build the first GPS sensor/client (browser location API)
- Scope: `shared-general`
- Board card: `t_3f4ea929`
- Evidence: https://github.com/EFForg/rayhunter/issues/1047
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1033 — Wifi Client option fails to activate, and makes web server inaccessible.
- Scope: `direct-orbic`
- Board card: `t_2f594a51`
- Evidence: https://github.com/EFForg/rayhunter/issues/1033
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1031 — A full release workflow
- Scope: `shared-general`
- Board card: `t_f581f5b7`
- Evidence: https://github.com/EFForg/rayhunter/issues/1031
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1017 — Add documentation about how to add a heuristic
- Scope: `shared-general`
- Board card: `t_195c1b2d`
- Evidence: https://github.com/EFForg/rayhunter/issues/1017
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1013 — shove unparsed 3g/2g traffic into pcap
- Scope: `shared-general`
- Board card: `t_44836819`
- Evidence: https://github.com/EFForg/rayhunter/issues/1013
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #1000 — Wifi based heuristics
- Scope: `shared-general`
- Board card: `t_27bf0438`
- Evidence: https://github.com/EFForg/rayhunter/issues/1000
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #990 — ROADMAP
- Scope: `shared-general`
- Board card: `t_26e082ff`
- Evidence: https://github.com/EFForg/rayhunter/issues/990
- Pseudocode: `read_current_docs_and_source_of_truth(); identify_stale_or_missing_steps(); write_minimal_canonical_section(); validate_commands_and_links(); run_diff_check();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #984 — Opt-in HTTP tap for the raw /dev/diag byte stream
- Scope: `direct-orbic`
- Board card: `t_ad85afc6`
- Evidence: https://github.com/EFForg/rayhunter/issues/984
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #920 — Orbic USB installer is becoming harder to maintain
- Scope: `direct-orbic`
- Board card: `t_ebe3a77e`
- Evidence: https://github.com/EFForg/rayhunter/issues/920
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #916 — Feature: optional keep-screen-on for Orbic
- Scope: `direct-orbic`
- Board card: `t_42f6a384`
- Evidence: https://github.com/EFForg/rayhunter/issues/916
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #914 — Feature: Add additional ui_level to show preset/custom PNGs for Low/Medium/High detections.
- Scope: `shared-general`
- Board card: `t_8817bd8d`
- Evidence: https://github.com/EFForg/rayhunter/issues/914
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #901 — [Installer] Orbic RC400L ORB400L_V1.2.8_BVZRT: orbic + orbic-usb both fail (telnet + AT+SYSCMD rootshell)
- Scope: `direct-orbic`
- Board card: `t_4550acbc`
- Evidence: https://github.com/EFForg/rayhunter/issues/901
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #880 — add on-device function to fully dis/enable wifi on the Orbic RC400L
- Scope: `direct-orbic`
- Board card: `t_9634fe70`
- Evidence: https://github.com/EFForg/rayhunter/issues/880
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #868 — We need more contributors
- Scope: `shared-general`
- Board card: `t_1931edfd`
- Evidence: https://github.com/EFForg/rayhunter/issues/868
- Pseudocode: `read_current_docs_and_source_of_truth(); identify_stale_or_missing_steps(); write_minimal_canonical_section(); validate_commands_and_links(); run_diff_check();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #833 — UZ801 displays Test Heuristic Warnings without SIM Card.
- Scope: `shared-general`
- Board card: `t_19eb5726`
- Evidence: https://github.com/EFForg/rayhunter/issues/833
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #807 — Unable to switch from teathered to wifi install
- Scope: `direct-orbic`
- Board card: `t_f932760f`
- Evidence: https://github.com/EFForg/rayhunter/issues/807
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #785 — M7350 initialization failed error code -1 test heuristic shows no errors
- Scope: `shared-general`
- Board card: `t_5c633ea2`
- Evidence: https://github.com/EFForg/rayhunter/issues/785
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #769 — PowerShell not showing up in contextual menu on Windows 11 Pro
- Scope: `direct-orbic`
- Board card: `t_7c384844`
- Evidence: https://github.com/EFForg/rayhunter/issues/769
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #762 — Keep orbic-usb installer alive
- Scope: `direct-orbic`
- Board card: `t_567716be`
- Evidence: https://github.com/EFForg/rayhunter/issues/762
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #756 — [feature] Add heuristic for Timing Advance abnormalities
- Scope: `shared-general`
- Board card: `t_6d62d9d5`
- Evidence: https://github.com/EFForg/rayhunter/issues/756
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #737 — How to uninstall using orbic-shell?
- Scope: `direct-orbic`
- Board card: `t_e2fd4edf`
- Evidence: https://github.com/EFForg/rayhunter/issues/737
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #736 — Proxmity/ Signal Source Finder Module
- Scope: `shared-general`
- Board card: `t_508f44cb`
- Evidence: https://github.com/EFForg/rayhunter/issues/736
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #733 — build-firmware-devel produces too large binaries for moxee
- Scope: `shared-general`
- Board card: `t_b82a8be2`
- Evidence: https://github.com/EFForg/rayhunter/issues/733
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #732 — EFF Logo bugs out during charging
- Scope: `direct-orbic`
- Board card: `t_f8ccd66e`
- Evidence: https://github.com/EFForg/rayhunter/issues/732
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #730 — rayhunter-check: can not read the PCAP file from QCSuper although no Encapsulation
- Scope: `shared-general`
- Board card: `t_a710ee6d`
- Evidence: https://github.com/EFForg/rayhunter/issues/730
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #719 — rayhunter-check not give the warning even if SIB7 appear
- Scope: `shared-general`
- Board card: `t_9a527334`
- Evidence: https://github.com/EFForg/rayhunter/issues/719
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #693 — Network installer fails on Orbic RC400L: "exit code 0" not found in: command done, exit code 1
- Scope: `direct-orbic`
- Board card: `t_9c5b0cea`
- Evidence: https://github.com/EFForg/rayhunter/issues/693
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #607 — Implement capture storage management in rayhunter-daemon
- Scope: `shared-general`
- Board card: `t_399d8615`
- Evidence: https://github.com/EFForg/rayhunter/issues/607
- Pseudocode: `trace_recording_lifecycle(); define_limits_and_failure_semantics(); add_persistence_and_corruption_tests(); implement_atomic_bounded_operation(); verify_redaction_and_recovery();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #591 — [Feat] Add steps to documentation for developing new devices
- Scope: `shared-general`
- Board card: `t_5c902206`
- Evidence: https://github.com/EFForg/rayhunter/issues/591
- Pseudocode: `read_current_docs_and_source_of_truth(); identify_stale_or_missing_steps(); write_minimal_canonical_section(); validate_commands_and_links(); run_diff_check();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #589 — feat: Turn Orbic WiFi from AP to Client mode
- Scope: `direct-orbic`
- Board card: `t_06b220a9`
- Evidence: https://github.com/EFForg/rayhunter/issues/589
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #557 — IMSI Requested with out attach - False positive.
- Scope: `shared-general`
- Board card: `t_711b55da`
- Evidence: https://github.com/EFForg/rayhunter/issues/557
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #543 — Disconnected after Identity Request without Auth Accept
- Scope: `direct-orbic`
- Board card: `t_ae44316f`
- Evidence: https://github.com/EFForg/rayhunter/issues/543
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #539 — Prevent screen shut off (when plugged in)
- Scope: `direct-orbic`
- Board card: `t_bd6dfd89`
- Evidence: https://github.com/EFForg/rayhunter/issues/539
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #534 — New criterions and features
- Scope: `shared-general`
- Board card: `t_115f3a40`
- Evidence: https://github.com/EFForg/rayhunter/issues/534
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #523 — Cant access Orbic via ADB after enabling tethering
- Scope: `direct-orbic`
- Board card: `t_73b43ef2`
- Evidence: https://github.com/EFForg/rayhunter/issues/523
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #510 — Installer v0.5.0 fails unhelpfully on full file system
- Scope: `direct-orbic`
- Board card: `t_98290123`
- Evidence: https://github.com/EFForg/rayhunter/issues/510
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #501 — Allow users to set display name and notes for rayhunter recordings
- Scope: `shared-general`
- Board card: `t_37132e68`
- Evidence: https://github.com/EFForg/rayhunter/issues/501
- Pseudocode: `model_power_and_charging_state(); reproduce_each_transition(); update_state_machine(); add_regression_fixture(); require_hardware_or_provider_ci_for_device_claims();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #480 — empty pcap and unhandled GsmRrSignallingMessage on PinePhone
- Scope: `shared-general`
- Board card: `t_14309480`
- Evidence: https://github.com/EFForg/rayhunter/issues/480
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #462 — Develop FlashCatch heuristic
- Scope: `shared-general`
- Board card: `t_22e23b7a`
- Evidence: https://github.com/EFForg/rayhunter/issues/462
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #457 — Support layer 2 MAC packets
- Scope: `shared-general`
- Board card: `t_3ec71cbd`
- Evidence: https://github.com/EFForg/rayhunter/issues/457
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #441 — Multiple ideas for reducing binary size
- Scope: `shared-general`
- Board card: `t_96829736`
- Evidence: https://github.com/EFForg/rayhunter/issues/441
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #398 — New possible indicators
- Scope: `shared-general`
- Board card: `t_404a1775`
- Evidence: https://github.com/EFForg/rayhunter/issues/398
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #377 — [Feature Request]: Move telcom parser to it's own crate
- Scope: `shared-general`
- Board card: `t_6493de12`
- Evidence: https://github.com/EFForg/rayhunter/issues/377
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #363 — [Feature Request] Add Count of Severity Level Warnings to UI
- Scope: `shared-general`
- Board card: `t_65be73ca`
- Evidence: https://github.com/EFForg/rayhunter/issues/363
- Pseudocode: `read_issue_and_current_tree(); trace_affected_symbols_and_callers(); add_failing_regression_fixture(); implement_minimal_change(); run_targeted_and_nearest_full_checks();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #326 — [Feature Request]: Record and show current and neighboring cells
- Scope: `shared-general`
- Board card: `t_89f5f754`
- Evidence: https://github.com/EFForg/rayhunter/issues/326
- Pseudocode: `define_privacy_and_permission_boundary(); model_unavailable_and_denied_states(); add_fixture_or_contract_test(); implement_opt_in_data_flow(); verify_no_secret_or_location_leakage();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #300 — [Feature Request]: Allow users to re-run analysis on old recordings
- Scope: `shared-general`
- Board card: `t_54ebbe21`
- Evidence: https://github.com/EFForg/rayhunter/issues/300
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #261 — [Feature Request]: A hardware-based CI testing rig for all supported devices
- Scope: `direct-orbic`
- Board card: `t_6ea00559`
- Evidence: https://github.com/EFForg/rayhunter/issues/261
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #259 — [Feature Request]: Orbic RC400L - change LTE bands for Europe
- Scope: `direct-orbic`
- Board card: `t_014606e0`
- Evidence: https://github.com/EFForg/rayhunter/issues/259
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #207 — Carrier unlock
- Scope: `direct-orbic`
- Board card: `t_b17be10e`
- Evidence: https://github.com/EFForg/rayhunter/issues/207
- Pseudocode: `discover_device_profile(); constrain(vendor, product, interface, serial); reproduce_in_disposable_fixture(); change_minimal_device_path(); assert_no_wrong_device_and_no_live_mutation();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #195 — Question on Dependency Security / Supply Chain Security
- Scope: `shared-general`
- Board card: `t_8f5dbfd0`
- Evidence: https://github.com/EFForg/rayhunter/issues/195
- Pseudocode: `inventory_targets_and_toolchains(); reproduce_gate_or_size_failure(); add_deterministic_gate_or_budget_check(); fix_without_broad_upgrade(); verify_reproducible_artifact_and_security_report();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #160 — CHOICE Additions not supported yet
- Scope: `shared-general`
- Board card: `t_9c39815b`
- Evidence: https://github.com/EFForg/rayhunter/issues/160
- Pseudocode: `trace_choice_schema_and_parser(); add_unsupported-choice_fixture(); define_forward-compatible_behavior(); implement_explicit_error_or_preservation(); test_round_trip();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #154 — Tool for nulling sensitive QMDL/PCAP fields
- Scope: `shared-general`
- Board card: `t_6f7e602f`
- Evidence: https://github.com/EFForg/rayhunter/issues/154
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #153 — Another indicator of IMSI catcher activity (compare public IP with announced IP ranges)
- Scope: `shared-general`
- Board card: `t_810e4e03`
- Evidence: https://github.com/EFForg/rayhunter/issues/153
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #139 — PerCodec:DecodeError:Requested Bits to decode 3, Remaining bits 1
- Scope: `direct-orbic`
- Board card: `t_47b6d7c4`
- Evidence: https://github.com/EFForg/rayhunter/issues/139
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #113 — Support for detecting silent SMS messages
- Scope: `shared-general`
- Board card: `t_1bfbac8f`
- Evidence: https://github.com/EFForg/rayhunter/issues/113
- Pseudocode: `locate_parser_and_analyzer_contract(); add_realistic_red_fixture(); emit_expected_detection_without_false_positive(); preserve_existing_analyzers(); run_targeted_then_full_parser_tests();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #108 — Telemetry
- Scope: `shared-general`
- Board card: `t_1384a2d9`
- Evidence: https://github.com/EFForg/rayhunter/issues/108
- Pseudocode: `define_privacy_and_permission_boundary(); model_unavailable_and_denied_states(); add_fixture_or_contract_test(); implement_opt_in_data_flow(); verify_no_secret_or_location_leakage();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #98 — Automated testing for RRC parser
- Scope: `shared-general`
- Board card: `t_8dd407f4`
- Evidence: https://github.com/EFForg/rayhunter/issues/98
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #81 — Include RSSI info in pcap
- Scope: `direct-orbic`
- Board card: `MISSING`
- Evidence: https://github.com/EFForg/rayhunter/issues/81
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #78 — Setting to upload results to S3 Bucket
- Scope: `shared-general`
- Board card: `MISSING`
- Evidence: https://github.com/EFForg/rayhunter/issues/78
- Pseudocode: `trace_recording_lifecycle(); define_limits_and_failure_semantics(); add_persistence_and_corruption_tests(); implement_atomic_bounded_operation(); verify_redaction_and_recovery();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

### #58 — web UI: download entire QMDL store as zip
- Scope: `shared-general`
- Board card: `t_23531757`
- Evidence: https://github.com/EFForg/rayhunter/issues/58
- Pseudocode: `trace_input_to_persistence_and_ui(); define_backward_compatible_data_contract(); add_fixture_for_current_failure(); implement_smallest_parser_or_storage_change(); verify_round_trip_and_sibling_formats();`
- Acceptance: reproduce/prove current behavior; cover changed symbols/callers and sibling-device effects; targeted tests pass or blocker is recorded; provider CI/hardware/UI evidence is separately labeled.
- Stop if: issue is stale/duplicate, required hardware or operator decision is unavailable, or ownership/lease is ambiguous.

## 6. Additional non-issue lanes

1. `origin/fix/orbic-serial-response-framing-901-reviewed`: inspect diff, serial framing, timeout/error semantics, wrong-device risk, tests, and frontend prerequisite; classify and stage only owned files.
2. `origin/fix/orbic-network-installer-693-reviewed`: inspect exit-status parsing and installer retry/transport boundaries; require device profile constraints and disposable fixture.
3. `origin/fix/orbic-adb-tethering-reconnect-523-reviewed`: audit the Qualcomm-vendor fallback as a safety blocker; require Orbic product/interface/serial constraints and regression tests.
4. Source TODO/FIXME sweep: re-scan tracked source, group markers by file territory, create idempotent cards only for evidenced actionable gaps, and never mix TODO cleanup with feature code.
5. Missing mappings from the snapshot: #81 RSSI-in-PCAP and #78 S3 upload require live re-query and idempotent card creation only if still in scope; do not silently omit them.

## 7. Batch and bridge rules

- Propose 3–5 highest-value units after pickup, but dispatch at most one active mutation unit.
- Every directive needs a unique `ack_token`, a Markdown file, a sibling `.pointer.yaml`, an exact required outbox path, and one concise channel notification. Archive superseded directives.
- Claude must not self-select a follow-on card. When a unit is complete, return the receipt and wait.
- Hermes independently verifies board, Git ancestry/status, changed-file ownership, receipt paths, and test logs before advancing.
- Stage only the exact owned files for Claude to commit; do not stage package-lock or coordination files unless the directive names them.
- If a worker reports Claw/Velociraptor context, immediately HOLD and send a scope correction; Rayhunter is the only valid project for this command.

## 8. Stop conditions

HOLD on missing/contradictory board identity, absent card lease, dirty-path ownership ambiguity, overlapping files, active compaction, missing RED/GREEN logs, filtered tests presented as full, hardware claims without hardware/provider evidence, UI work without attended evidence, credential/account decisions, live-server risk, unexpected commit/push, or any source change outside the declared lease. A heartbeat is liveness only; it is not completion.

## 9. Final drain report

Report fresh counts; issue-to-card reconciliation; dispositions by class; completed/held/staged commits; local/provider/hardware/UI evidence; all blockers and operator decisions; exact next unit; and `RELEASE_READINESS: NOT_CLAIMED` unless every applicable gate independently passes.
