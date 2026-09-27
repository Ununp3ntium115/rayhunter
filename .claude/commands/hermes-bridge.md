# Rayhunter Hermes Bridge

Use this command to synchronize the current Rayhunter Claude Code session with Hermes/Evey.
This command is for **Rayhunter / EFForg rayhunter only**. Do not substitute Velociraptor Claw Edition paths, boards, or release rules.

## Required scope

- Repository: current repository root (`git rev-parse --show-toplevel`)
- Expected repo: `/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan`
- Hermes board: `rayhunter-orbic`
- Board commands must set `HERMES_KANBAN_BOARD=rayhunter-orbic` explicitly.
- Hermes is the controller/verifier; Claude is the bounded worker.
- Preserve existing dirty and untracked paths. Do not reset, clean, stash, overwrite, commit, push, or merge unless a directive explicitly authorizes it.

## Step 0 — read authoritative context

Read these before accepting work:

1. `CONTRIBUTING.md`
2. `.hermes/context/rayhunter-ai-contribution-policy.md`
3. `CLAUDE.md` if present
4. `.hermes/context/rayhunter-orbic-review-20260925-session-01a0da5b-context.md` if present
5. `.hermes/context/rayhunter-backlog-map-20260925.md` if present
6. `.hermes/context/claude-orbic-issue-feature-sweep-prompt.md` if present

If a directive conflicts with live repository state, report the conflict instead of guessing.

## Bridge contract

Bridge root:

```text
~/.hermes/claude-bridge/
```

- `inbox/`: Hermes directives (`.md` plus `.pointer.yaml` when present)
- `outbox/`: Claude result receipts; do not delete them
- `active/`: receipts already read by Hermes
- `channel.jsonl`: short liveness and ACK messages only

For every directive with an `ack_token`:

1. Read the complete directive and its pointer.
2. Append exactly one JSON line to `channel.jsonl` with `from: "claude-code"`, `to: "evey"`, and the exact token.
3. Write the exact required outbox receipt named by the directive.
4. Include commands, exit codes, changed files, evidence paths, blockers, and next action.
5. Do not claim completion from a chat message alone.

Use this message shape:

```json
{"from":"claude-code","to":"evey","type":"ack","ack_token":"<exact token>","message":"Accepted Rayhunter directive; reading required context."}
```

## Live board readback

Before selecting a card, run:

```bash
HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban stats
HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban list --status running
HERMES_KANBAN_BOARD=rayhunter-orbic hermes kanban list --status ready
```

Do not infer card ownership from another board, stale channel history, or a process alone. A card must identify the Rayhunter workspace, exact owned files, verification commands, and stop conditions.

## Work rules

- One bounded workstream at a time.
- No mass-unblocking or self-selected follow-on card.
- Keep Orbic-specific work separate from shared/general Rayhunter work.
- Hardware claims require hardware/provider-CI evidence; local tests are not hardware proof.
- Build the frontend before daemon Rust checks: `npm install && npm run build -w daemon/web`.
- Preserve unrelated `package-lock.json` changes.
- Do not identify Orbic from Qualcomm vendor ID `0x05c6` alone.
- Run focused verification before broader suites.
- Run `codegraph sync .` after accepted source or documentation changes when the directive permits it.
- AI-generated changes must be understood and tested by the maintainer. Do not post AI-generated public comments or PR descriptions. Before a PR handoff, fill every applicable checklist item from the repository template. Public commits and pull requests must be authored by `ununp3ntum115` and nobody else; do not add AI/automation co-authors or trailers. Commit messages must be human-approved, diff-accurate, imperative, and free of AI/automation claims.

## Result receipt

Write the exact required outbox file using this structure unless the directive specifies another schema:

```markdown
# Rayhunter result: <task-id>

## Summary
- What was changed or verified

## ack_token
<exact token>

## Commands and exit codes
- `<command>` — exit `<code>`

## Files changed
- `<path>` or `none`

## Evidence paths
- `<absolute or repo-relative path>`

## Blockers
- `None`, or the exact unresolved blocker

## Next action
- One bounded recommendation; do not self-select another card
```

## No-directive behavior

If there is no valid Rayhunter directive, do not invent work. Report the live board and repository state, then wait.
