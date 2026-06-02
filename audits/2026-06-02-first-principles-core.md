# fragments — first-principles pass (ceo directive 2026-06-02)

_Relay `20260602203540478189`: "reason about your domain from first principles, find the fundamental truths, then consider how a minimal unopinionated core + connectors for opinionated capabilities can improve what you're working on."_

## The domain, stripped to atoms

The problem fragments exists for: **the same content must live in many files, and copies drift.** It's DRY violated across boundaries tooling can't span — different files, different formats, sometimes different repos.

The conventional fix is **transclusion** (includes, partials, template engines, SSI): one source, generated outputs. But transclusion bakes in opinion — a build step, a runtime, a templating dialect — and the deployed artifact is *generated*, so the consuming file is not valid or editable on its own.

fragments takes the other branch: **physical copy + marker-guarded reconciliation.** The content is physically present in each target, wrapped in a comment in the target's *native* syntax. A tool overwrites the marked region from the canonical fragment. Copies that can't drift because a tool reconciles them — not a single source compiled away.

## Fundamental truths

1. **The target file is the artifact, not a byproduct.** It stays valid, complete, and hand-editable in its own format with zero build step. This is the whole reason to choose marker-sync over transclusion — and it forbids any HTML/format opinion in the core.
2. **Four irreducible operations:** a *fragment* (named canonical content), a *marker* (native-syntax comment delimiting a named region), *sync* (replace region ← fragment), *check* (detect drift without writing — the CI/idempotency gate). Nothing else is fundamental.
3. **The canonical source is recoverable, so the target is disposable.** A torn or stale target region is always reconstructable by re-running sync. This is why durability of the write path is not load-bearing (see below).
4. **Everything format- or policy-specific is opinion** and belongs outside the core.

## Minimal core / connector split — does the lens reveal an improvement?

Mapping every current surface against the lens:

| Surface | Mechanism (core) or opinion (out)? | Placement | Verdict |
|---|---|---|---|
| region replace by named marker | mechanism | core | ✓ correct |
| native comment syntax per extension (`src/syntax.rs`) | mechanism + *data* default, `[syntax]`-overridable | core | ✓ correct (default ≠ opinion when overridable) |
| `exclude_dirs` | policy | config (empty default) | ✓ routed out |
| per-target transform (`SyncHook`) | opinion | consumer | ✓ routed out |
| anything HTML | opinion | pagekit | ✓ routed out |
| `--json` / stdout formatting | presentation | binary layer, not lib | ✓ routed out (v0.8.0 lib-purity) |
| variables / partials / nesting / reverse-sync | opinion / scope creep | deferred — no consumer pulls | ✓ correctly deferred |

**Conclusion: the lens reveals no core change.** This is not a coincidence — it's the same result the three-track dogfood reached empirically this morning (`production-readiness-bar.md`: zero `[CORE]` items). The architecture is already the faithful expression of the domain's fundamental truths. Reasoning top-down (first principles) and bottom-up (dogfood) converge on the same answer, which is the strongest signal the split is right.

## The one place the lens actually tests

The accepted decision "direct `fs::write` (truncation risk accepted; recovery = sync re-run + check)" is the only spot where a robustness shortcut sits in the core. Under first principles it is **correctly accepted, not a debt**: truth #3 says the target region is always reconstructable from the canonical fragment, so a torn write is self-healing on the next sync and no source data can be lost. Adding atomic write-temp-rename would be mechanism-pure (no opinion cost) but buys only "don't have to re-run sync after a crash mid-write" — marginal against a self-healing path. Leaving it is consistent with KISS and minimal-core. Flagged below as the single optional refinement, not a recommendation.

## Net

No action on core. fragments is the minimal unopinionated primitive the directive describes; the connector/config/consumer split is faithful. The directive is satisfied by confirmation-with-proof, not by change.
