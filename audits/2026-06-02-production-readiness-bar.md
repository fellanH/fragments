# fragments — core production-readiness bar

_2026-06-02. Defines what "production-ready" means for a **minimal mechanism core** (per `behavior/minimal-core-connectors.md`), and audits fragments-sync v0.8.0 against it. The point of this document: production-readiness for a primitive is **robustness + a sharp contract + a small error surface**, NOT more capabilities. Every gap below is routed `[CORE]` (rare — fix in core) or `[CONNECTOR]` (almost always — opinion belongs outside)._

## The bar

A dimension is `PASS` (meets bar), `GAP` (real shortfall, needs work), or `OPEN` (needs a decision before it can be scored). Tracks 1 (format-agnostic dogfood) and 2 (consumer-friction harvest) feed evidence into the `OPEN` rows.

| # | Dimension | Bar | v0.8.0 status | Route |
|---|-----------|-----|---------------|-------|
| 1 | **Contract stability** | Public surface is small, documented, semver-disciplined | PASS — `sync_all`/`_with`, `check_all`/`_with`, `watch::run`/`_with`, `Config`, `SyncHook`, `CommentSyntax`, `--json` schemas. Spec §"Library API" current. | — |
| 2 | **Idempotency** | Re-running sync changes nothing; writes only on byte-diff | PASS — byte-comparison write; alphabetical iteration; `doctor` output sorted (v0.8.0). | — |
| 3 | **Error surface** | Few, named, self-describing failure modes | **PASS** (Track 1) — only 2 ways to break (hand-edit self-heals; malformed marker → `check`/`doctor`). Dogfood confirmed unpaired / duplicate-name / orphan errors name the file + reason and are actionable. | — |
| 4 | **Write durability** | Mid-write crash is recoverable; trade is documented | PASS-by-design — `fs::write` truncate, recovery = idempotent re-run, preserves inode/perms/xattrs. Documented in spec §"Write durability". Sub-question resolved: **no consumer pulls for atomic-write** (Track 2 found none) → core stays minimal. | — |
| 5 | **Observability** | Machine-readable output; library doesn't pollute consumer stdout | PASS — `--json` on check/list/doctor (v0.8.0, re-verified on clean build); library stdout purity shipped (v0.8.0). | — |
| 6 | **Failure isolation** | One malformed target doesn't silently corrupt or abort the whole run unpredictably | **PASS** (Track 1) — `sync` makes **no writes** while a malformed/duplicate condition exists; `check`/`doctor` report it. Behavior is intentional and safe (fail-clean, not partial-corrupt). | — |
| 7 | **Format-agnostic path proven** | The v0.7.0 headline feature has ≥1 real non-HTML consumer | **CLOSED** (Track 1) — dogfood synced one fragment into markdown + YAML + CSS + SQL simultaneously, each valid in native syntax, idempotent on re-run. First production-shaped validation of v0.7.0. Still want a *standing* non-HTML consumer (the AGENTS.md rollout) to keep it exercised. | dogfood, done |
| 8 | **Cross-root story** | Clear, documented answer for "sync a shared block across sibling project roots" | **RESOLVED** (Track 1) — verdict: **symlink the canonical `_fragments/<name>` into each repo** (the `docs-as-fragments` pattern), or one `target_dir` over a parent with `max_depth`. **No core change.** Action: document this in spec §Patterns ([CONNECTOR]/DOCS). | [CONNECTOR]/docs |
| 9 | **Test coverage** | Core paths + each format family + hooks + json covered | PASS-ish — 46 tests (integration + format_agnostic + hooks + json_output). Confirm format_agnostic suite exercises each comment-style family. | — |
| 10 | **Distribution** | Published, licensed, reproducible binaries | PASS — `fragments-sync` v0.8.0 on crates.io, dual MIT/Apache, tagged-release CI for cross-platform binaries. | — |
| 11 | **Spec/docs convergence** | Spec matches code; no drift | PASS — `specs/fragments.md` reconverged through v0.8.0; AGENTS.md rule refs fixed 2026-06-02. | — |

## Reading (post-Track-1/2, 2026-06-02)

- **All 11 dimensions PASS / CLOSED / RESOLVED.** The format-agnostic GAP (#7) closed by dogfooding, not by adding features. The three OPENs (#3 message quality, #6 failure isolation, #8 cross-root) all resolved with **zero core code change** — exactly the healthy outcome for a minimal mechanism core.
- **No `[CORE]` work emerged.** Every finding across both tracks routed `[CONNECTOR]` (pagekit defaults/guidance), `[CONFIG/CONVENTION]`, or `[DOCS]`. The one finding initially tagged `[CORE]` (`--json` missing) was a stale-binary artifact — retracted. The other (pagekit CSS-asset link false-positives) is pagekit's HTML link-integrity layer, not fragments core.
- **The core is production-ready as a primitive.** Remaining work is connector/docs hardening, not core change. The triage gate held: dogfooding generated zero pressure to bloat the core, and the deferred features (variables, partials, nesting, reverse-sync) stayed deferred — no consumer proved the sync-only model insufficient.

## Net actions falling out of this audit (all outside core)

1. **[CONNECTOR] pagekit `exclude_dirs` defaults** — collapse the 7–11 item list copy-pasted across 5 sites into a pagekit built-in. Highest leverage, ~30m. *(pagekit seat — relay.)*
2. **[DOCS] spec §Patterns: cross-repo symlink** — document "symlink `_fragments/<name>` from a canonical parent" as the official multi-repo answer. *(fragments seat — small, can do here.)*
3. **[CONNECTOR] pagekit "when to use raw fragments vs pagekit" decision note** — we-know-aeo + stormfors KB bypass pagekit without guidance. *(pagekit seat.)*
4. **[OPTIONAL] standing non-HTML consumer** — roll the shared AGENTS.md "Canon rules" block out as a real synced fragment (cross-workspace, symlink pattern) to keep bar #7 continuously exercised. Felix-gated (crosses workspace scope).

## Triage gate (apply to every Track 1/2 finding)

1. Is it **mechanism** (how sync reads/writes/reports) or **policy/opinion** (what to sync, what to exclude, domain conventions)? Mechanism → consider core. Policy → connector/config.
2. Default verdict is **[CONNECTOR]**. A `[CORE]` route requires naming the mechanism only the core can provide cleanly.
3. Record the finding, route, and rationale. A finding routed `[CORE]` ships behind verify + a test; `[CONNECTOR]` ships in pagekit or a per-project `fragments.toml`.
