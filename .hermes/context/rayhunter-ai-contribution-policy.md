# Rayhunter AI contribution and publication policy

This repository's `CONTRIBUTING.md` is authoritative. Read it before preparing commits, pull requests, issue comments, review comments, or release notes.

Publication rule

- Do not post AI-generated GitHub issue comments, pull-request comments, review comments, or discussion replies.
- Do not use an AI tool to impersonate the human author in a public description.
- AI may help inspect code, draft private notes, organize evidence, or prepare a human-reviewable draft, but the maintainer must rewrite/review and publish the final prose.
- If an upstream template asks the author to claim that no generative-AI tools were used, never make that claim. Stop at a private draft and request human handling.
- Do not create, edit, close, label, assign, merge, or publish GitHub objects without exact operator authorization for the target.

Commit-message rule

Public commits and pull requests must be authored by `ununp3ntum115` and nobody else. Do not add Claude, Hermes, an AI tool, automation, co-author trailers, or alternate author identities. AI may propose a private commit message, but the human maintainer owns the final message and must ensure it accurately describes the diff. Use a concise imperative subject, identify the actual scope, and do not mention AI or unverifiable testing claims. Never use a commit message as a substitute for a PR description or checklist.

Before commit or handoff

- Read `CONTRIBUTING.md` and the relevant issue/PR template.
- Verify the diff, ownership of every changed path, and the exact tests actually run.
- Keep generated or coordination-only files separate from product changes unless explicitly requested.
- Do not claim hardware, provider-CI, UI/UA, or release evidence that was not obtained.
- Record AI assistance privately in the internal receipt when required; do not add an AI-generated public comment.

Pull-request checklist

Before submitting a PR, fill out the repository's PR template checklist completely. Do not leave unchecked boxes that are applicable. If no PR template is present, prepare a private draft checklist covering:

- What changed and why.
- Related issue or discussion.
- Tests run with exact commands and outcomes.
- `cargo fmt` and `cargo clippy` status where applicable.
- Manual testing for new features, including device/UI scope and limitations.
- Documentation/configuration changes.
- Security, privacy, and sensitive-data review.
- Hardware evidence, provider CI, and UI/UA evidence separately.
- Known limitations, follow-up work, and reviewer decisions.
- Confirmation that the final public description/comments were written and approved by the human maintainer.

Agent instruction

When asked to publish or comment, respond internally with a private evidence packet and stop before the public mutation unless the operator explicitly authorizes a human-reviewed final text. Keep the packet factual, concise, and free of subscriber identifiers, exact GPS, credentials, and raw capture payloads.
