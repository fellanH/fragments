# Relay → pagekit seat (2026-06-02)

From the fragments seat. Source of items 1–3: `../audits/2026-06-02-consumer-friction.md` (full per-consumer evidence + routing). All items are **pagekit / config / docs scope — none touch fragments core.** Low priority (tooling refinement), no deadline.

## Items

1. **[HIGHEST LEVERAGE, ~30m] Ship pagekit with hardcoded `exclude_dirs` defaults.** Identical 7–11 item `exclude_dirs` lists are copy-pasted across 5 sites — `ettsmart.se`, `phoeberyanofficial.com`, `felixhellstrom.com`, `felixhellstrom.se`, `weknowaeo.com`. fragments core ships `exclude_dirs = []` by design (config over convention); the static-site default set (`css`, `fonts`, `_assets`, `dist`, `build`, `node_modules`, …) is exactly the opinion that belongs in pagekit's config layer. Collapse the copy-paste into a pagekit built-in.

2. **Add a "when to use raw `fragments` vs `pagekit`" decision note.** `we-know-aeo` and `stormfors/knowledge-base` bypass pagekit and drive the raw `fragments` binary, losing pagekit's best-practice guidance. A short decision tree in pagekit docs would route consumers correctly.

3. **Include `[[fragments]]` inventory metadata in the pagekit project template.** `we-know-aeo` and `stormfors/knowledge-base` lack a declared fragment inventory; the pagekit scaffold/template should include it.

## Bonus (same seat, cheap — not in the audit)

4. **Fix pagekit's `AGENTS.md` stale rule-citation prefix.** It carries the same `harness/rules/...` prefix I fixed in fragments today (fragments commit `0c1a4d3`). Correct it to `omni-os/omni-system/packages/rules/`, and **drop the two non-existent refs** `build-not-dev.md` and `dispatch-verification.md` (they exist in no rules tree; their intent is covered by `cargo build --release` guidance + `valuable-deliverable.md`).

## Context

These fell out of a three-track fragments dogfood (Felix-requested, tooling-refinement focus). Verdict: fragments **core** is production-ready as a minimal primitive — zero core changes needed; all friction routes outward to connectors, which is what minimal-core predicts. These four items are the connector/config/docs half of that.
