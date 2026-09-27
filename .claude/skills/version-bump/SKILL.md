---
name: version-bump
description: Bump the version across all rayhunter crates and the frontend in one coordinated step
disable-model-invocation: true
---

# Version Bump

Ask for the new version number (e.g. `0.14.0`).

Then:

1. Run `./scripts/set-versions.sh <VERSION>` — this updates all `Cargo.toml` crates and the frontend `package.json` in one pass.

2. Run `cargo check` to confirm nothing broke.

3. Verify the version appears correctly:
   ```bash
   grep -r "^version = " */Cargo.toml Cargo.toml | grep -v "^Binary"
   grep '"version"' daemon/web/package.json
   ```

4. Stage and commit:
   ```bash
   git add -p  # review changes before staging
   git commit -m "chore: bump version to <VERSION>"
   ```

5. Report: which files changed, what versions they now contain, and the commit SHA.

## Notes
- Do NOT manually edit individual `Cargo.toml` or `package.json` version fields — `set-versions.sh` is the authoritative way to keep them in sync.
- `Cargo.lock` will update automatically on the next `cargo build` — that's expected.
- The release workflow (`release.yml`) is triggered by pushing a tag: `git tag v<VERSION> && git push origin v<VERSION>`.
