---
name: rayhunter-hermes-bridge
description: Use when coordinating Claude with Hermes for Rayhunter work.
version: 0.1.0
author: Brody Nielsen, Hermes Agent
license: MIT
platforms: [macos, linux, windows]
metadata:
  hermes:
    tags: [rayhunter, orbic, claude-code, hermes, bridge, kanban]
    related_skills: []
---

# Rayhunter Hermes Bridge

Use this skill when Claude is coordinating with Hermes/Evey on the Rayhunter repository. It defines the Rayhunter-only board, bridge receipt, dirty-tree, hardware-evidence, and verification boundaries. It does not authorize source edits, card mutation, builds, commits, pushes, or hardware actions by itself.

## When to use

- A user asks Claude to run `/hermes-bridge` or synchronize with Hermes.
- A Hermes directive arrives for Rayhunter Orbic/shared-feature work.
- A task needs a board readback, ACK token, outbox receipt, or ownership check.

## Required context

Read `CLAUDE.md` and any present files under `.hermes/context/` that are named by the directive. The repository is Rayhunter/EFForg rayhunter, not Velociraptor Claw Edition.

## Board contract

Always use the explicit board environment:

```bash
HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban stats
HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban list --status running
```

Do not infer ownership from another board, stale bridge history, or a process cwd alone. Select only a directive naming this repository, exact file territory, verification commands, and stop conditions.

## Bridge contract

Bridge root is `~/.hermes/claude-bridge/`. Read complete inbox directives. For an exact `ack_token`, append one `from: "claude-code"` JSON line to `channel.jsonl` and write the exact required outbox receipt. Never delete outbox receipts or treat a heartbeat as completion proof.

## Rayhunter rules

- Preserve existing dirty and untracked files.
- Do not reset, clean, stash, overwrite, commit, push, or merge unless explicitly authorized.
- Keep Orbic-specific and shared/general changes distinct.
- Do not infer Orbic from Qualcomm USB vendor `0x05c6` alone.
- Build `daemon/web` before daemon Rust checks.
- Distinguish local, provider-CI, and physical-hardware evidence.
- Run focused verification before broad suites.
- Do not select a follow-on card after completing a directive.

## Verification

A valid handoff includes the exact task ID and ACK token, commands with exit codes, files changed, evidence paths, blockers, and one bounded next action. If no valid directive exists, report live board and repository state without inventing work.
