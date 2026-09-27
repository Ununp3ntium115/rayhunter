# SDLC Plan — Rayhunter Orbic IMPLEMENT_NOW Fixes

## Context

Rayhunter is an open-source IMSI-catcher detector for ARM-based LTE hotspots (EFForg/rayhunter).
This plan covers the three IMPLEMENT_NOW items from the triage pass conducted on 2026-09-25.
All work is LOCAL-ONLY. Do not push branches or create GitHub PRs without explicit operator approval.
All commits are on isolated branches; never commit to main.

## Global Constraints

- **No upstream mutation**: Never push, create PRs, create issues, or comment on GitHub
- **No hardware**: Never run ADB, telnet, or reboot commands against physical devices
- **One fix per commit**: Each task produces one clean, atomic commit
- **Pristine code**: `cargo fmt`, `cargo clippy`, `cargo test` must all pass before commit
- **No `??=` operator**: The repo bans it (see `b8c4c0c` — `Forbid ??= in the codebase`)
- **No `eslint-disable max-lines`** or oxlint-disable max-lines
- **Comments**: Only add comments for non-obvious WHY; no explanatory HOW comments
- **Type assertions**: Avoid except `as const`; unavoidable casts need `// SAFETY:` comment
- **Cross-platform**: Use `path::join` not hardcoded separators; no hardcoded `e.metaKey`

## Tasks

### Task 1: Fix #1129 — Remove erroneous 5G band row from doc/orbic.md

The Orbic RC400L is an LTE-only modem. `doc/orbic.md` incorrectly lists 5G NR bands
(n260/n261, n77, n2/5/48/66) in the Supported Bands table. This misleads users
into thinking the device supports 5G, which it does not.

**Target file**: `doc/orbic.md`

**Change**: Delete line 18 — the row `| 5G (wideband,midband,nationwide)  | n260/n261, n77, n2/5/48/66 |`

**Branch**: Create `fix/doc-orbic-5g-bands-1129` from `main`

**Verification**:
1. `mdbook test` — must pass
2. Semgrep scan on changed file

**Commit message**:
```
doc: remove erroneous 5G NR band row from orbic.md

The RC400L is an LTE-only modem. The 5G NR row (n260/n261, n77,
n2/5/48/66) was incorrect and would mislead users evaluating the
device for 5G use.

Fixes #1129

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

**Expert agent**: No specialized agent needed (pure Markdown doc fix)

---

### Task 2: Verify fix/orbic-serial-response-framing-901-verified

Branch `fix/orbic-serial-response-framing-901-verified` fixes a serial framing bug
in `installer/src/orbic.rs`: the original code assumed exactly 2 USB bulk reads
(one for echo, one for response); the fix replaces this with a deadline-based loop
that accumulates packets until a terminal modem response (`OK` or `ERROR`) is found,
via a new `SerialResponseStatus` enum.

**Goal**: Confirm the branch:
1. Passes `cargo fmt --check -p installer`
2. Passes `cargo clippy -p installer -- -D warnings`
3. Passes `cargo test -p installer`
4. Has no Semgrep findings in the changed files

**Branch to check out**: `fix/orbic-serial-response-framing-901-verified`
(already exists locally; create an isolated worktree for testing)

**Report**: Collect exact test output. If tests pass, report VERIFIED. If tests fail,
investigate root cause and report the failure output.

**Expert agent**: `rust-expert` + `hotspot-engineer`

---

### Task 3: Verify fix/orbic-network-installer-693-verified

Branch `fix/orbic-network-installer-693-verified` fixes a cross-device remount bug:
`setup_rayhunter` previously always ran `mount -o remount,rw /dev/ubi0_0 /`,
which is only needed for Moxee. The fix adds a `remount_root: bool` parameter;
Orbic passes `false`, Moxee retains `true`.

**Goal**: Confirm the branch:
1. Passes `cargo fmt --check -p installer`
2. Passes `cargo clippy -p installer -- -D warnings`
3. Passes `cargo test -p installer`
4. Has no Semgrep findings in the changed files
5. Confirm: `orbic_network::install` is called with `false` from `lib.rs`
6. Confirm: `moxee::install` still calls `setup_rayhunter` with `true`

**Branch to check out**: `fix/orbic-network-installer-693-verified`
(already exists locally; create an isolated worktree for testing)

**Report**: Collect exact test output. Report VERIFIED or FAILED with details.

**Expert agent**: `rust-expert` + `hotspot-engineer`
