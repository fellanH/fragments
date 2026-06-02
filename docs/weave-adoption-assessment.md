# weave-site-model adoption — assessment (DECLINE, surfaced for decision)

_Task `tas-8fb92245` (coordinator GREEN dispatch): "Phase 1 weave site-model adoption — replace fragments' private page/asset derivation with weave-site-model, kills duplication #1." Spec: `~/omni/products/clean-clone/docs/core-vs-opinion.md` §5. **Read-only review done; refactor declined pending Felix decision. Reason below — every claim verified against code.**_

## Verdict

**fragments should not depend on `weave-site-model`.** The dispatch rests on a premise that does not hold for this repo, and adopting site-model would inject HTML-website opinion into a deliberately format-agnostic primitive — violating `minimal-core-connectors`, the rule weave itself cites. There is also a hard publishability blocker. Detail:

## 1. fragments has no page/asset derivation — the "3× dup" miscounts it

The dispatch says "replace your private page/asset derivation." fragments' entire file discovery is `collect_target_files` (`src/sync.rs:185`): a `WalkDir` that returns `Vec<PathBuf>` of **every text file** under the target dir (filtered by `exclude_dirs` + `max_depth` + the fragments dir). There is **no `Page`, no `Asset`, no `AssetKind`, no `LinkGraph`, no DOM, no `<title>`, no "is this a page" predicate.** It does not parse HTML. It lists files of *any* format (`.html`, `.css`, `.sql`, `.yaml`, `.md`, `.toml`) and resolves comment syntax per extension.

The spec's "site-model derived 3×" (§2.1) is real for **clean-clone** (`inventory::is_page`) ↔ **freedom** (`content_layout::discover`) — both genuinely derive an HTML page model. fragments is counted as the third via "implicit file-scan," but that is a **category error**: a generic "list files with markers" walk is a *different, more general* mechanism, not a third copy of "what is a page." Killing it removes no duplication — fragments shares no code or concept with the other two.

## 2. site-model is HTML-website-shaped → adopting it imports opinion into the format-agnostic core

`weave-site-model` (verified at `weave/crates/site-model/src/lib.rs`):
- `AssetKind { Html, Css, Js, Image, Font, Other }` — an HTML-clone taxonomy. fragments' real formats (`.sql`, `.yaml`, `.toml`, `.sh`, `.md`) **all collapse to `Other`**, which is exactly the distinction fragments needs (it keys comment syntax off the extension). site-model is *lossy* for fragments' job — it gives fragments nothing and would force re-deriving the extension anyway.
- `Page { rel, title }` + `pages()` reads each file and extracts `<title>` via `scraper` (an HTML DOM parser); `is_page` encodes "HTML doc not under fragments dir." fragments needs none of this. Depending on site-model drags **`scraper` + an HTML-page concept into the minimal format-agnostic primitive** — the precise contradiction v0.7.0 removed (`tasks/arc.md`: "a primitive that bakes in HTML opinionation isn't a primitive") and that `minimal-core-connectors` forbids.

## 3. It is not behavior-preserving — two real regressions

- **`max_depth` lost.** fragments bounds its walk by `config.max_depth`; `Site::discover` / `DiscoverOptions` has **no depth bound**. Adoption changes which files sync.
- **Exclude semantics differ.** fragments excludes by **path prefix** (`config.exclude_dirs`, `p.starts_with(ex)`); weave skips by **directory name anywhere** (`skip_dirs` on `file_name`). Different files included/excluded.

The spec calls Phase 1 "behavior-preserving... guarded by check gates." For fragments it is neither behavior-preserving nor net-zero.

## 4. Hard blocker: it would make fragments unpublishable

fragments ships on crates.io as **`fragments-sync` v0.8.0** (`cargo install fragments-sync` works today). `weave` is a **private** repo (`github.com/fellanH/weave`, spec line 181). A published crates.io crate **cannot depend on a private git repo** — publish and `cargo install` both fail to resolve it. Adoption would either break the public crate or force fragments off crates.io. This alone blocks the move regardless of the design merits.

## 5. The spec's own routing is ahead of itself

The same document, read in full, does not authorize this refactor now:
- §3 table (line 136): **`fragments` → "re-exported from `compose` (already a clean generic primitive — minimal change)."** It assigns fragments to the **compose** stage, *not* site-model.
- §5 progress note (lines 182-184): fragments adopting site-model is **relayed to the revenue-system coordinator to sequence**; "this seat [clean-clone] owns weave core + the consumer contracts only."
- §5 header (line 166) and §7 (line 220): "**This document authorizes none of them — it is the RED proposal.** ... Awaiting Felix's go/no-go before any code moves."

So a direct "refactor now" dispatch to fragments runs ahead of the proposal's own RED status and routing.

## Recommendation

1. **fragments stays the standalone format-agnostic primitive at the `compose` seam** (matching §3), with no weave dependency. The fragments↔suite contract remains the published lib API (see `docs/core-vs-opinion.md`).
2. **Drop duplication #1's fragments leg** — it isn't one. The real kill is clean-clone ↔ freedom, both true page-model derivers; that proceeds without fragments.
3. **If there is generality worth sharing, the arrow points the other way:** weave's `discover` could *gain* fragments' `max_depth` + prefix-exclude generality. That is a change to **weave** (owned by clean-clone), relayed to its owner — and still would not make fragments depend on weave (crates.io constraint stands).
4. **Decision is Felix's** — this conflicts with a Felix-greenlit effort, so it's surfaced, not unilaterally closed.
