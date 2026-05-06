# fragments

Marker-region sync for any text format with comment-pair syntax. 40 tests passing.

## Active arc

**Stage 1 of fragments → pagekit fork.** Reframing fragments as a format-agnostic primitive; HTML-specific helpers (`init`, `extract`, framework profiles) move to a new [`pagekit`](../pagekit) workspace.

Stage 1 (this commit): documentation only. Spec preamble rewritten, AGENTS.md neutralized, default `exclude_dirs` cleared (config-over-convention). Code unchanged — `init` and `extract` still ship in the fragments binary.

Stage 2 (next): code split. `lib.rs` in fragments exposes public API; `init.rs`/`extract.rs` move into pagekit; `[[extract.candidates]]` config moves with them.

Stage 3 (later): pagekit's `extract` migrates from `scraper` to `lol_html` (cleaner source-rewrite, no attribute-normalization hack).

felixhellstrom.com remains the canonical real-site consumer for the HTML use case. Will migrate to the pagekit binary once Stage 2 ships.

## Decisions

- Format-agnostic primitive. No HTML-specific opinionation in fragments core.
- pagekit owns HTML/website-specific surface; depends on fragments crate.
- Default `exclude_dirs` is empty — config over convention.
- Direct `fs::write` (truncation risk accepted; recovery is sync re-run + check).
- Single-binary CLI in pagekit re-exposes fragments commands; agent UX is one binary, one CLI.

## Open questions

(none open)

## Resolved

- Default `fragments_dir = "_fragments"` (underscore prefix). Resolved 2026-05-06 — Felix confirmed all sites in his stack use the underscore convention so static-site hosts (CF Pages, Eleventy, Jekyll) treat the folder as infrastructure and skip it during deploy. Was previously `fragments` per spec; flipped to match consumer practice.

## Backlog

- **Stage 2** (code split): `lib.rs` in fragments exposes core APIs; `init.rs`/`extract.rs` move to pagekit; pagekit binary builds and tests.
- **Stage 3** (lol_html): rewrite `extract` on `lol_html` in pagekit.
- **Far future** (only if a non-HTML consumer pulls): comment-syntax-per-extension config so `/* */`, `# `, `// `, etc. work natively without setting `marker_prefix`.

## Blocked

Nothing.
