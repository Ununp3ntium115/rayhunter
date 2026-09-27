# Rayhunter Orbic review handoff context

## Source provenance

- Session ID: `01a0da5b-dfb9-7883-846f-c21d1742a23a`
- Raw transcript: `/Users/brodynielsen/.rayhunter-backups/conversation-exports/rayhunter-orbic-review-20260925-session-01a0da5b-dfb9-7883-846f-c21d1742a23a.jsonl`
- Checksum file: `/Users/brodynielsen/.rayhunter-backups/conversation-exports/rayhunter-orbic-review-20260925-session-01a0da5b-dfb9-7883-846f-c21d1742a23a.jsonl.sha256`
- Verified size: `5,630,668` bytes
- Verified SHA-256: `6944e962a7a0eb06b7b158cf96c995e1637549be3b73f3a9e55fa53b5f955b30`
- Export contains 533 JSONL records. It is a Codex review session, not a Claude session.

## Review objective from the source session

Read-only review of the synced EFForg/rayhunter Orbic contribution work. The reviewed pushed branches were:

- `fix/orbic-serial-response-framing-901-reviewed`
- `fix/orbic-network-installer-693-reviewed`
- `fix/orbic-adb-tethering-reconnect-523-reviewed`

The source review was instructed to fetch `upstream/main`, inspect each branch for correctness, wrong-device risk, cross-device regressions, tests, and CI readiness, and not publish or mutate upstream. GitHub PR creation was noted as blocked by repository interaction restrictions.

## Verified/intermediate findings to carry forward

- The review worktree started clean, with `origin` and `EFForg/rayhunter` (`upstream`) remotes; no repository-local `AGENTS.md` was found.
- All three named branch heads existed and contained the fetched `upstream/main` tip (`e9d92012…`).
- Formatting passed on all three heads.
- The ADB branch has a high-risk device-selection issue: its fallback treats USB vendor `0x05c6` as Orbic-specific. `0x05c6` is Qualcomm's vendor ID and can match unrelated devices, so this is a wrong-device/safety blocker until constrained by Orbic-specific product/interface/serial evidence.
- A raw workspace `cargo check --verbose` initially failed because the fresh detached worktree lacked the required generated frontend assets under `daemon/web/build/*`; this matches the repository's documented prerequisite ordering. The changed installer compiled before that asset failure. The correct follow-up is `npm install && npm run build -w daemon/web`, then repeat Rust gates.
- A package-lock modification appeared in the primary worktree during the source review and was preserved as unrelated drift; do not overwrite or clean it without explicit scope.
- Device evidence narrowed separate Orbic Wi-Fi follow-up facts outside the three branch diffs: firmware has `wpa_supplicant 2.4` (rejects SAE fields), and hidden WPA2 profiles require `scan_ssid=1`. Upstream issue #1033 already covers Orbic Wi-Fi client failure; do not duplicate it or attribute these facts to the installer branches.
- The source review found the fork's Issues disabled and declined to publish an upstream issue because the repository bug template requires the submitter to assert that no generative-AI tools were used; that assertion could not honestly be made.
- Firmware artifact evidence was intentionally limited: one usable raw image, `mtd0-sbl.bin`, size `2,621,440` bytes, SHA-256 prefix/suffix reported as `62f407…03aa`; reads of `mibib`, `tz`, `rpm`, `aboot`, `boot`, `scrub`, `misc`, and `recovery` timed out or errored. Sensitive partitions (`efs2`, `dynamic_nv`, `efs2_bak`, `modem`, `sec`) were excluded. Earlier 26-byte archive files were invalid shell-error artifacts.
- The source review stopped an active CI check during handoff. Do not report full CI readiness or a final pass from this export alone; the transcript ends before a completed structured final report.

## Operating constraints for Claude

Treat the raw JSONL as evidence and this file as a concise index, not as permission to mutate upstream, touch connected hardware, publish issues/PRs, or overwrite unrelated worktree drift. Before acting, re-check live git status, branch/remotes, and current files. Separate source-session observations from fresh verification. If continuing the review, run the repository's documented gates in an isolated worktree and record exact commands/results; distinguish local proof from provider CI.
