# fragments

Marker-region sync. v0.8.0 (library purity + `--json`; format-agnostic comment syntax from v0.7.0). 46 tests passing across integration + hooks + format_agnostic + json_output suites.

## Active arc

Production-hardening pass shipped (2026-06-02). The fragments → pagekit fork is fully shipped (Stages 1-3); fragments is the format-agnostic primitive, pagekit is the HTML site management layer.

v0.7.0 makes the "format-agnostic" framing *true in code*. Until now the implementation was hardcoded HTML-only (`.html` filter + `<!-- -->` markers) despite the docs/CLI claiming any-format support — the central contradiction undermining the fork's whole rationale (a primitive that bakes in HTML opinionation isn't a primitive). Now markers are ordinary comments in the target file's own format, resolved per extension via a built-in table (`src/syntax.rs`) that's extensible through `[syntax]` config. One fragment syncs into HTML, CSS/JS, YAML/shell, SQL, etc. — each file valid in its native syntax. The high-level lib API (`sync_all_with`/`check_all_with`/`watch::run_with`/`Config`/`SyncHook`) is unchanged, so pagekit is unaffected; `referenced_fragment_names` gained a `CommentSyntax` arg (pagekit doesn't use it).

Also in this pass: dual MIT/Apache licensing + crates.io metadata, tagged-release CI workflow (cross-platform binaries), CHANGELOG, duplicate-fragment-name detection, panic-hardened name derivation, and doc reconvergence (README/AGENTS rewritten; boot path corrected to products/ not workspaces/, command list 6 not 8).

**Released:** `v0.7.0` tag pushed; GitHub Release is live with linux-x86_64 + macOS arm64/x86_64 binaries attached (initial run failed on a missing `contents: write` permission — fixed in the workflow, re-fired, green). **Published to crates.io 2026-06-02 as [`fragments-sync` v0.7.0](https://crates.io/crates/fragments-sync)** (the bare `fragments` name was squatted; binary/CLI command stays `fragments`).

v0.6.0 added the `SyncHook` API for per-target fragment transforms. v0.6.1 closed a gap surfaced by pagekit Sprint 4 D2: `watch::run_with(hooks)` mirrors `sync_all_with`/`check_all_with`, so reactive resyncs honor the same hook stack as initial sync.

## Decisions

- Format-agnostic primitive. No HTML-specific opinionation in fragments core.
- pagekit owns HTML/website-specific surface; depends on fragments crate.
- Default `exclude_dirs` is empty — config over convention.
- Default `fragments_dir = "_fragments"` — underscore prefix signals to deploy hosts.
- Direct `fs::write` (truncation risk accepted; recovery is sync re-run + check).
- Single-binary CLI in pagekit re-exposes fragments commands; agent UX is one binary, one CLI.
- Per-target transforms live in fragments via `SyncHook`, not in consumers via pre-derivation. Pagekit's pull triggered the call (Sprint 4 D2). Argument was that "the same fragment, adapted for where it goes" is naturally part of sync; the hook API validates that against a real consumer immediately rather than waiting for a hypothetical second one.
- Watch must hook (v0.6.1). Initial-sync-only hooking would have meant reactive resyncs silently produced different output. Surfaced by pagekit D2's commit body flagging it as a known gap; closed in fragments rather than worked around in pagekit.

## Open questions

(none open)

## Resolved

- Default `fragments_dir = "_fragments"` (underscore prefix). Resolved 2026-05-06 — Felix confirmed all sites in his stack use the underscore convention so static-site hosts (CF Pages, Eleventy, Jekyll) treat the folder as infrastructure and skip it during deploy. Was previously `fragments` per spec; flipped to match consumer practice.
- Comment-syntax-per-extension. Resolved 2026-06-02 (v0.7.0) — built-in extension→syntax table in `src/syntax.rs` plus `[syntax]` config overrides. `/* */`, `# `, `-- `, etc. work natively. Pulled forward from "far future" because the docs/CLI already claimed format-agnosticism the code didn't deliver; the gap was a correctness/credibility bug, not a feature request.
- **Library purity + `--json` (QoL pass).** Resolved 2026-06-02 — (1) the library no longer writes to stdout: `sync_all`/`sync_all_with` kept their `usize` return (pagekit unaffected) but the per-file print moved to the `fragments` binary; new `sync_all_paths`/`sync_all_paths_with` return the updated `Vec<PathBuf>`. Fixes leaked progress lines into consumer (pagekit) output. (2) `--json` on `check`/`list`/`doctor` with stable `kind`-tagged schemas + `collect()` report functions (`CheckReport`/`ListReport`/`DoctorReport`); exit codes unchanged. 46 tests (was 43; +3 `tests/json_output.rs`). Backward-compatible — pagekit's 112 tests pass against it. Declined scaffolding/nested-subdirs/reverse-sync/UI (no consumer pulling). `serde_json` added as a dep. **Released as [`fragments-sync` v0.8.0](https://crates.io/crates/fragments-sync)** (2026-06-02, Felix-gated publish) — minor bump for additive features; tag `v0.8.0` pushed.
- **crates.io publish.** Resolved 2026-06-02 — published as [`fragments-sync` v0.7.0](https://crates.io/crates/fragments-sync). The bare `fragments` name was squatted (abandoned v0.1.0 registered 2021-07-29, not actually unclaimed as the prior baton assumed), so the crate ships as `fragments-sync` while the binary/CLI command and `use fragments::` lib name stay `fragments`. Felix configured the token and verified his crates.io email; `cargo install fragments-sync` now works.

## Relayed (out of scope here)

- **pagekit connector/config/docs items (2026-06-02).** Four findings from the fragments dogfood — `exclude_dirs` defaults, raw-vs-pagekit decision note, `[[fragments]]` template metadata, AGENTS.md rule-prefix fix — relayed to the pagekit seat via feed `dec-1b5f0b5b` + brief at `docs/relay-to-pagekit-2026-06-02.md`. None touch fragments core. The cross-workspace synced-fragment idea (shared AGENTS.md block) was **deferred by Felix** (coordination cost vs KISS; revisit later).

## Backlog

- **Far future**: line-comment block fragments spanning multiple comment lines (current line-comment markers wrap a region, which already covers the common case). Nested fragments (deferred — see `specs/fragments.md`).

## Blocked

(none)
