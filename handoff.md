# fragments — handoff baton

_Written 2026-06-02 at rotation checkpoint. Seat: fragments worker. HEAD `fafa181`, tree clean, `main` == `origin/main`._

## TL;DR for the next agent

fragments is **production-ready, published, and live on crates.io as [`fragments-sync`](https://crates.io/crates/fragments-sync)**. Two things happened this session beyond the v0.7.0 release: (1) the crates.io publish completed (the prior baton's one open item), and (2) a small QoL pass shipped (library purity + `--json`). There is **one open item**, and it's a Felix gate, not a blocker on you: a minor version bump + `cargo publish` to ship the QoL changes (the token is now saved and the crates.io email is verified, so it's literally one command). Canonical state lives in `tasks/arc.md`.

## The one open item — republish QoL changes (Felix-gated, ~one command)

- The working tree on `main` (HEAD `fafa181`) has unreleased changes (library purity + `--json`). crates.io still serves **v0.7.0**, which predates them.
- To ship: bump `version` in `Cargo.toml` (these are additive features → **0.8.0**), add a CHANGELOG `[0.8.0]` heading (content already drafted under `[Unreleased]`), tag, and `cargo publish`.
- Auth is **no longer a blocker**: the crates.io token is saved (`~/.cargo/credentials.toml`) and Felix verified his crates.io email this session. `cargo publish` is irreversible/public, so it stays Felix's call — but it's now a single command, not a setup task.
- Tracked in `tasks/arc.md` → the QoL Resolved entry notes "Unreleased on crates.io."

## What shipped this session

**1. crates.io publish (the prior baton's blocked item).** Published as `fragments-sync v0.7.0`. The bare `fragments` name was **squatted** (abandoned v0.1.0 from 2021-07-29 — the prior baton wrongly assumed it was unclaimed). Crate name is `fragments-sync`; the **binary/CLI command and `use fragments::` lib name stay `fragments`** (`[[bin]]`/`[lib] name = "fragments"`). So `cargo install fragments-sync` installs a `fragments` command. Felix supplied the token + verified email mid-session.

**2. QoL pass (`fafa181`), both backward-compatible:**
- **Library purity.** `sync_all`/`sync_all_with` used to `println!` each updated path from *inside the library*, leaking progress lines into consumer (pagekit) stdout. That formatting moved to the `fragments` binary. Both keep their `usize` return (pagekit unaffected). New `sync_all_paths`/`sync_all_paths_with` return the updated `Vec<PathBuf>` for callers wanting the list.
- **`--json`** on `check`/`list`/`doctor` for agent/CI consumers — stable `kind`-tagged schemas via new `collect()` functions and serializable report types (`CheckReport`, `ListReport`/`FragmentRef`, `DoctorReport`/`DoctorIssue`). Exit codes unchanged. `doctor` orphan-marker output is now deterministically sorted. Added `serde_json` dep.

## Verification done (don't redo blindly)

- `cargo fmt --check`, `clippy -D warnings`, `cargo package` — all clean.
- **46 tests pass** (27 integration + 9 format_agnostic + 7 hooks + 3 new `tests/json_output.rs`).
- **Consumer safe:** `pagekit`'s **112 tests pass** against `fafa181` (verified by building+testing pagekit against the on-disk fragments). The QoL change is additive; `list_fragments`/`run_doctor` signatures are unchanged.

## pagekit coordination (resolved — don't re-touch)

- pagekit and fragments share **one on-disk directory** via a path-dep. The crate rename to `fragments-sync` broke pagekit's `fragments = { path = "../fragments" }` (cargo resolves path deps by *package name*).
- The **pagekit seat owns and already fixed this** — commit `1b67de8 chore: adopt fragments-sync package name` adds `package = "fragments-sync"` to pagekit's dep (the dep *key* stays `fragments`, so pagekit's `use fragments::` is unchanged). **Do not edit pagekit's Cargo.toml from this seat** — it's their scope, and it's done.
- pagekit independently added its own `--json`/report work (`2bab3de`) in parallel — unrelated to fragments' `--json`.

## Do NOT do

- Don't re-fire/re-tag the v0.7.0 GitHub Release or re-publish v0.7.0 to crates.io — both are live and correct.
- Don't invent backlog. Deferred-by-design items in `tasks/arc.md` (nested fragments, multi-line block fragments, reverse sync) and declined QoL (scaffolding, nested fragment subdirs, colored UI) have **no consumer pulling** — respect `subtract-before-building`. Scaffolding specifically belongs to pagekit by the fork boundary.
- Don't touch pagekit's repo — separate seat, separate scope.

## Minor flag (not acted on)

`AGENTS.md` still references rule paths under `harness/rules/...` which may be a stale prefix. Left untouched — wasn't confident of the correct mapping and it's tangential. Confirm against canon before relying on those paths.
