# fragments — handoff baton

_Written 2026-06-02 (autonomy flywheel checkpoint). Seat: fragments worker. HEAD `0c1a4d3`, tree clean, `main` == `origin/main`._

## TL;DR for the next agent

fragments is **production-ready, published, and live on crates.io as [`fragments-sync` v0.8.0](https://crates.io/crates/fragments-sync)**. The prior baton's one open item — republishing the QoL changes (library purity + `--json`) — **is now shipped** (Felix-gated publish executed this session). **There are no open items.** Canonical state lives in `tasks/arc.md`.

## What shipped this session

**1. v0.8.0 release (closes the prior open item).** Bumped `0.7.0 → 0.8.0` (minor — additive features), moved the CHANGELOG `[Unreleased]` block to `[0.8.0] — 2026-06-02`, committed (`2866e9c`), tagged `v0.8.0`, pushed, and ran `cargo publish`. Live on crates.io. The `Release` GitHub Actions workflow fired on the tag to attach cross-platform binaries. Pre-publish gate: `cargo fmt --check` clean, `clippy -D warnings` clean, **46 tests pass**.

**2. AGENTS.md rule-path reconverge (`0c1a4d3`).** The `harness/rules/...` prefix was stale — real rules live under `omni-os/omni-system/packages/rules/` (indexed via `~/.claude/rules/INDEX.md`). Fixed the prefix on the 3 references that resolve (`omni/tier-architecture`, `workflow/valuable-deliverable`, `workflow/subtract-before-building`). **Dropped 2 dangling citations** — `build-not-dev.md` and `dispatch-verification.md` exist in no rules tree; their intent was already covered locally (`cargo build --release` in the build section; "verify by running the binary" folded into the valuable-deliverable line).

## Carried-forward facts (still true)

- Crate name is `fragments-sync` (bare `fragments` was squatted); **binary/CLI command and `use fragments::` lib name stay `fragments`**. `cargo install fragments-sync` installs a `fragments` command.
- Library purity: `sync_all`/`sync_all_with` keep their `usize` return (pagekit unaffected); the per-file print lives in the binary; `sync_all_paths`/`sync_all_paths_with` return the `Vec<PathBuf>`.
- `--json` on `check`/`list`/`doctor` — stable `kind`-tagged schemas via `collect()` + serializable report types. Exit codes unchanged.

## Do NOT do

- Don't re-publish v0.7.0 or v0.8.0, or re-fire their releases — all live and correct.
- Don't invent backlog. Deferred-by-design items in `tasks/arc.md` (nested/multi-line block fragments, reverse sync) and declined QoL (scaffolding, nested subdirs, colored UI) have **no consumer pulling** — respect `subtract-before-building`. Scaffolding belongs to pagekit by the fork boundary.
- Don't touch pagekit's repo — separate seat, separate scope.

## Flag for the pagekit seat (not this seat's scope)

pagekit's `AGENTS.md` carries the **same stale `harness/rules/...` prefix** (and references the same two non-existent rules). Relay to the pagekit seat for the equivalent reconverge; do not fix it from here.
