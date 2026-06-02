# fragments — handoff baton

_Written 2026-06-02 at rotation checkpoint. Seat: fragments worker. HEAD `cec77b3`, tree clean, `main` == `origin/main`._

## TL;DR for the next agent

fragments is **production-ready and shipped at v0.7.0**. There is exactly **one open item**, and it's blocked on Felix, not on you: publishing to crates.io (needs a registry token). Don't invent other work — the arc is genuinely drained except for deferred-by-design backlog. Canonical state lives in `tasks/arc.md`; this baton just points at the one live thread.

## The one open item — crates.io publish (BLOCKED on Felix)

- **Status:** package is publish-ready (`cargo package` passes clean). The only blocker is auth — no crates.io token exists on the machine (`~/.cargo/credentials.toml`, legacy file, and `CARGO_REGISTRY_TOKEN` env all absent).
- **Felix authorized FULL PUBLISH** earlier this session (tag + GitHub Release done; crates.io is the remaining half).
- **To finish the moment a token exists:**
  ```bash
  cd ~/omni/products/fragments
  # Felix runs: cargo login   (token from https://crates.io/settings/tokens)
  cargo publish               # then you run this
  ```
  The `fragments` crate name appeared unclaimed. This is an irreversible/public action — already authorized, but it requires Felix's token, so it stays parked until he provides one.
- Also tracked in `tasks/arc.md` → **Blocked**.

## What shipped this session (v0.7.0)

The headline: fragments **claimed** "format-agnostic" everywhere but was hardcoded HTML-only (`.html` filter + `<!-- -->` markers). Now true in code.

- **Format-agnostic comment syntax** — `src/syntax.rs` maps file extension → comment delimiters (SGML `<!-- -->`, C-family `/* */`, hash `#`, dash `--`); `[syntax]` config extends/overrides it. One fragment syncs into HTML/CSS/JS/YAML/shell/SQL/Markdown, each valid in its native format. Fragment name = file stem; duplicate stems error.
- **Hardening:** line-comment name-boundary safety (`nav` ≠ `navbar`), no-panic name derivation, duplicate-name detection.
- **Distribution:** dual `MIT OR Apache-2.0` (both LICENSE files), Cargo.toml metadata, CHANGELOG.md, `.github/workflows/release.yml` (tagged cross-platform binaries; `checkout@v5`).
- **Released:** `v0.7.0` GitHub Release is **live, draft=false, 3 binaries attached** (linux-x86_64, macOS arm64/x86_64). Initial release run failed on a missing `contents: write` permission — fixed in the workflow and re-fired green.
- **Docs reconverged to match the binary:** `README.md`, `AGENTS.md`, `tasks/arc.md`, and the 405-line `specs/fragments.md` (it had documented `init`/`extract` as fragments commands — those moved to pagekit in the Stage-2 fork — and listed format-agnostic as "Future").

## Verification done (don't redo blindly)

- `cargo fmt --check`, `clippy -D warnings`, `cargo package` — all clean.
- **43 tests pass** (27 integration + 7 hooks + 9 new `tests/format_agnostic.rs`).
- Smoke-tested the installed binary on a real mixed-format dir (one fragment → HTML + CSS + YAML).
- **Consumer safe:** `pagekit` (path-dep at `../pagekit`) builds and its **107 tests pass** against v0.7.0. The high-level lib API (`sync_all_with`/`check_all_with`/`watch::run_with`/`Config`/`SyncHook`) is unchanged; only `referenced_fragment_names` gained a `CommentSyntax` arg (pagekit doesn't use it).

## Do NOT do

- Don't re-fire or re-tag the v0.7.0 release — it's live and correct.
- Don't invent backlog work. Remaining items in `tasks/arc.md` (nested fragments, multi-line block fragments, reverse sync) are **deferred by design** with no consumer pulling — respect `subtract-before-building`.

## Minor flag (not acted on)

`AGENTS.md` still references rule paths under `harness/rules/...` which may be a stale prefix. Left untouched — wasn't confident of the correct mapping and it's tangential. Confirm against canon before relying on those paths.
