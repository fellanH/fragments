# fragments

Marker-region sync. v0.6.0 (SyncHook API). 34 tests passing across integration + hooks suites.

## Active arc

Idle on fragments core. The fragments → pagekit fork is fully shipped (Stages 1-3); fragments is the format-agnostic primitive, pagekit is the HTML site management layer. Both consumers sync clean.

v0.6.0 just added the `SyncHook` API for per-target fragment transforms. Driven by pagekit Sprint 4 D2 (path-relative sync transforms for ettsmart.se's per-depth relative paths). pagekit will be the first real consumer of the hook API; their integration is the validation pass.

## Decisions

- Format-agnostic primitive. No HTML-specific opinionation in fragments core.
- pagekit owns HTML/website-specific surface; depends on fragments crate.
- Default `exclude_dirs` is empty — config over convention.
- Default `fragments_dir = "_fragments"` — underscore prefix signals to deploy hosts.
- Direct `fs::write` (truncation risk accepted; recovery is sync re-run + check).
- Single-binary CLI in pagekit re-exposes fragments commands; agent UX is one binary, one CLI.
- Per-target transforms live in fragments via `SyncHook`, not in consumers via pre-derivation. Pagekit's pull triggered the call (Sprint 4 D2). Argument was that "the same fragment, adapted for where it goes" is naturally part of sync; the hook API validates that against a real consumer immediately rather than waiting for a hypothetical second one.

## Open questions

(none open)

## Resolved

- Default `fragments_dir = "_fragments"` (underscore prefix). Resolved 2026-05-06 — Felix confirmed all sites in his stack use the underscore convention so static-site hosts (CF Pages, Eleventy, Jekyll) treat the folder as infrastructure and skip it during deploy. Was previously `fragments` per spec; flipped to match consumer practice.

## Backlog

- **Far future** (only if a non-HTML consumer pulls): comment-syntax-per-extension config so `/* */`, `# `, `// `, etc. work natively without setting `marker_prefix`.

## Blocked

Nothing.
