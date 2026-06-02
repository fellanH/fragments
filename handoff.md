# fragments — handoff baton

_Written 2026-06-02 (rotation checkpoint after dogfood pass). Seat: fragments worker. Tree clean, `main` == `origin/main`._

## TL;DR for the next agent

fragments is **production-ready, published as [`fragments-sync` v0.8.0](https://crates.io/crates/fragments-sync)**, and now **dogfood-validated**: a three-track audit (Felix-requested) confirmed the core is production-ready *as a minimal primitive* with **zero core changes needed** — every piece of friction routed outward to connectors/config/docs, exactly as minimal-core predicts. **No open items on this seat.** Canonical state: `tasks/arc.md`. Audits: `audits/2026-06-02-*.md`.

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
- Don't touch pagekit's repo — separate seat; its items are relayed, not yours.
- Don't action the relayed pagekit items or the deferred cross-workspace fragment from here.
