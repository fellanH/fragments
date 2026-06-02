# fragments — handoff baton

_Updated 2026-06-03 (after suite first-principles + customer-surface polish; HEAD `4cfc8ca`). Seat: fragments worker. Tree clean, `main` == `origin/main`._

## TL;DR for the next agent

fragments is **production-ready, published as [`fragments-sync` v0.8.0](https://crates.io/crates/fragments-sync)**, **dogfood-validated**, and now **suite-anchored**: it is the website-suite's format-agnostic primitive at the **compose** seam, and its published CLI contract (`--json` `ok:bool` + exit-1-on-findings) was ratified as the **suite machine-readable standard** — pagekit aligns to fragments, not the reverse. **No open items on this seat.** Canonical state: `tasks/arc.md`. Audits/docs: `audits/2026-06-02-*.md`, `docs/core-vs-opinion.md`, `docs/weave-adoption-assessment.md`.

## Session 2 (2026-06-03) — suite first-principles + polish

All Felix/coordinator-relayed, all resolved, zero open items:
- **First-principles / minimal-core (ceo directive):** `audits/2026-06-02-first-principles-core.md` + `docs/core-vs-opinion.md` — top-down reasoning converges with the dogfood: fragments owns the `fragment-sync` stage + the `SyncHook` **transform seam**; it is already the faithful minimal core, no extraction needed. The fragments→pagekit seam (4 stable lib signatures + lib-purity + hook-parity) is documented there.
- **weave-site-model adoption — DECLINED & resolved-at-coordinator:** `docs/weave-adoption-assessment.md`. fragments has no page/asset model (`collect_target_files` is a generic any-format walk, not a 3rd page-deriver); weave-site-model is HTML-shaped and a **published crate can't depend on the private weave repo** (breaks `cargo install`). fragments stays standalone. (If sharing ever makes sense it inverts: weave gains fragments' `max_depth`+prefix-exclude — relayed to clean-clone.)
- **Customer-surface polish (`4cfc8ca`):** fixed `--help` dir naming (`fragments/`→`_fragments/` ×3), broken crates.io README link, +7 missing format-table extensions. 46 tests green.
- **Fleet pitch (parked 7/8):** shared-blocks/marker-sync upsell via pagekit — `~/.omni/idea-queue/pitch-fragments-1.md`; Felix product call, blocked by distribution hold.
- **Toolchain gotcha:** cargo incremental fingerprints went stale (reported "Finished" without recompiling despite newer source). `cargo clean -p fragments-sync` fixed it; plain rebuild and sandbox-disable did not.

## What shipped this session

1. **v0.8.0 release** — library purity + `--json` on check/list/doctor; published to crates.io; tag `v0.8.0`; Release workflow attached binaries.
2. **AGENTS.md reconverge** (`0c1a4d3`) — fixed stale `harness/rules/` → `omni-os/omni-system/packages/rules/`; dropped 2 non-existent rule refs.
3. **Three-track dogfood** (`6083300`), artifacts in `audits/`:
   - `production-readiness-bar.md` — 11-dimension gate; all PASS/CLOSED/RESOLVED, no `[CORE]` work emerged. The triage gate held.
   - `format-agnostic-dogfood.md` — validated v0.7.0 non-HTML sync (md+yaml+css+sql) end-to-end; failure isolation safe; cross-root = symlink pattern. (One `--json` finding was a stale-binary artifact → retracted; verified `--json` on all 3 cmds in v0.8.0.)
   - `consumer-friction.md` — ~16 live HTML consumers; all friction → pagekit/config, none → core.
   - Spec gained a "syncing across sibling repos" §Patterns subsection.

## Open items

**None on this seat.** Two things live elsewhere:
- **Relayed to pagekit** (feed `dec-1b5f0b5b`, brief `docs/relay-to-pagekit-2026-06-02.md`): exclude_dirs defaults (highest leverage ~30m), raw-vs-pagekit note, `[[fragments]]` template, AGENTS.md prefix fix. Don't action from this seat.
- **Deferred by Felix:** rolling the shared AGENTS.md "Canon rules" block out as a standing cross-workspace synced fragment (coordination cost vs KISS — revisit later).

## Do NOT do

- Don't re-publish v0.7.0/v0.8.0 or re-fire releases — all live and correct.
- Don't add core features. The deferred items (variables, partials, nesting, reverse-sync) stay deferred — the dogfood confirmed no consumer needs them. Any new opinion belongs in pagekit/config, not the core (`minimal-core-connectors`).
- **Don't change the published `--json` schemas or exit codes** (`ok:bool`, exit-1-on-findings) — they are now the ratified **suite machine-readable standard**; altering them breaks the public contract *and* the suite. pagekit aligns to fragments here.
- **Don't add a `weave` dependency / adopt weave-site-model** — declined with cause (crates.io blocker + format-agnostic violation), resolved at coordinator. fragments stays the standalone compose-seam primitive.
- Don't touch pagekit's repo — separate seat; its items are relayed, not yours.
- Don't action the relayed pagekit items or the deferred cross-workspace fragment from here.
