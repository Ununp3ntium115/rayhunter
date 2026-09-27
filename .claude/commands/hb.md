# `/hb` — Rayhunter hourly control-plane heartbeat

Run one read-only state-aware heartbeat for `/Users/brodynielsen/GitRepos/Orca/rayhunter-orbic-work/leviathan`. Rayhunter only; never inspect or mutate Velociraptor Claw work. Read `CONTRIBUTING.md` and `.hermes/context/rayhunter-ai-contribution-policy.md`; do not generate or post public issue/PR/review comments. Public commits and pull requests must be authored by `ununp3ntum115` only.

- Read the collaborative drain prompt and exhaustive prompt's identity, pickup, bridge, fork-PR/branch inventory, and stop-condition sections.
- Verify git status/HEAD/recent log, active Claude PIDs/cwds, bridge channel tail, newest Rayhunter outbox receipt, and explicit-board Kanban stats plus running/ready lists.
- Compare with the previous heartbeat record; if no material state changed, return exactly `[SILENT]`.
- If state changed, report the delta and independently verify it. Do not stage, edit, build, test, scan, commit, push, claim cards, unblock the parent, or send a replacement directive unless a current explicit lease/directive authorizes that exact action.
- If Claude is on the wrong project, lacks a lease, or has unexplained dirty/committed paths, send one concise scope/hold message through the bridge and record evidence.
- If Claude is actively advancing under a valid lease, observe without interruption. A process alone is not proof; require bridge/outbox/Git evidence. If a PR is being prepared, verify the applicable repository checklist is filled before any publication request.
- Classify `ADVANCING`, `WAITING_RESULT`, `WAITING_DECISION`, `BLOCKED`, `SCOPE_DRIFT`, `COMPLETE_PENDING_VERIFY`, or `NO_CHANGE`.
- Keep credentials and sensitive device data out of output.
