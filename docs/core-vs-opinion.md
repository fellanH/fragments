# fragments — core vs. opinion (suite first-principles review)

_Felix directive (ceo relay, 2026-06-02): review this seat's mechanism-vs-opinion line against the suite's shared-core stages **site-model → fragment-sync → transform → compose → emit**. This seat owns **fragment-sync** and the **transform seam**. Read-only review + proposal — **no refactor** (that would be a RED call). Symbols below are the live v0.8.0 lib surface._

## TL;DR

fragments is already split mechanism-vs-opinion at the right line — the work was done incrementally (Config externalization, v0.8.0 lib purity, the `SyncHook` seam). This review re-derives that line against the *suite* vocabulary and finds **one latent thing worth naming, not refactoring**: the `fragments` **binary** is a connector wearing the same crate as the core, and the default syntax table is "sensible-default opinion" that happens to ship in-core. Both are fine where they are; naming them keeps the seam honest as the suite core forms.

## The line

**MECHANISM (belongs in the shared unopinionated core — the `fragment-sync` stage):**

- The sync engine: parse named markers in a file's *native* comment syntax → replace the marked region from canonical fragment content. `sync_all` / `sync_all_with` / `sync_all_paths` / `sync_all_paths_with` (`src/sync.rs`).
- Drift detection: `check_all` / `check_all_with` → `CheckIssue` (stale / malformed / duplicate). The CI/idempotency gate. (`src/sync.rs`)
- Format-agnostic syntax *resolution* mechanism: `syntax::resolve` + `syntax_key` — "given a path, find its comment delimiters." (`src/syntax.rs`)
- The reactive loop mechanism: `watch::run` / `run_with` — same engine, triggered on change. (`src/watch.rs`)
- The **transform seam**: the `SyncHook` trait (`src/sync.rs:137`). This is the mechanism for *the suite's `transform` stage* — "apply a per-target function during sync." The seam is core; the function body is opinion (lives in the consumer).
- Reference extraction: `referenced_fragment_names(…, CommentSyntax)` — "which fragments does this page cite." (`src/sync.rs:531`)
- `Fragments` loader — "load canonical fragment source." (`src/sync.rs:9`)

**OPINION (this seat's specific sync policy = a connector):**

- **Default conventions baked as Config defaults:** `fragments_dir = "_fragments"` (the underscore is a deploy-host-skipping opinion), `marker_prefix`, `target_dir`, `exclude_dirs`, `max_depth` (`src/config.rs`). Mechanism reads them; the *defaults are opinion*, and they're already overridable via `fragments.toml`.
- **The built-in extension→syntax table** (`builtin_syntax`, `src/syntax.rs:85`): the *contents* (`.html`→`<!-- -->`, `.css`→`/* */`, `.sql`→`-- `, …) are sensible-default opinion shipped in-core for zero-config usability. The *lookup* is mechanism; the *table* is opinion, overridable via `[syntax]`.
- **The entire `fragments` binary** (`src/main.rs`, `list.rs`, `doctor.rs` presentation, the `--json` schemas, all stdout): CLI ergonomics and presentation. This is a *connector over the core lib*, not core. v0.8.0 already drew this line — the lib no longer prints; `sync_all_paths` returns `Vec<PathBuf>` and the binary owns the printing.
- **Direct `fs::write`** (truncation-accepted) — a policy choice, validated as correct in `audits/2026-06-02-first-principles-core.md` (target region is reconstructable from the fragment, so a torn write self-heals).

## What I'd extract to a shared suite core

Almost nothing needs *moving* — the `fragments` **crate's lib API** already *is* the suite's `fragment-sync` stage plus the `transform` seam. If a shared suite core is formalized, the contribution from this seat is:

| Shared-core stage | fragments contributes | already-pure? |
|---|---|---|
| `fragment-sync` | `sync_all_with`, `check_all_with`, `watch::run_with`, `syntax::resolve` | ✅ yes (v0.8.0 lib purity) |
| `transform` (seam only) | `SyncHook` trait — the hook *point*, not any hook | ✅ yes |

No new extraction work. The one *clarifying* move (optional, not now): if/when the suite core wants a single config struct, split `Config` into `EngineConfig` (mechanism knobs the engine truly needs) vs `PolicyDefaults` (the `_fragments` / exclude conventions) — that would make the opinion explicitly a connector layer instead of default fields on the core struct. **Proposal only; do not do it without a consumer pulling for it (none today).**

## What stays this seat's connector

- The `fragments` **binary** — CLI, `--json` report schemas (`CheckReport` etc.), all human/agent presentation.
- The **default convention set** (`_fragments`, empty `exclude_dirs`, default marker prefix) and the **built-in syntax table contents** — sensible defaults, shipped for zero-config UX, fully overridable.
- File-discovery walking policy (`max_depth`, exclude handling).

## The fragments → pagekit coupling seam (this seat owns it)

Per suite relay `20260602084030`: **pagekit composes fragments core.** The seam *is* the `fragments` crate public API. Contract this seat owns and must not break without coordinating:

- **Stable signatures:** `sync_all_with(root, &Config, &[Box<dyn SyncHook>])`, `check_all_with(…)`, `watch::run_with(…)`, `referenced_fragment_names(…, CommentSyntax)`. pagekit calls these directly.
- **Lib purity invariant:** the lib must never write to stdout (broken once → fixed v0.8.0; pagekit's output stays clean).
- **Hook-stack parity:** every entry point that syncs must honor the same `SyncHook` stack — sync *and* watch (the gap pagekit's Sprint 4 D2 surfaced; closed v0.6.1). A new sync entry point that ignored hooks would silently diverge reactive output.
- **Direction of opinion:** pagekit supplies the `transform` bodies (HTML-specific hooks) and owns `site-model` / `compose` / `emit`. fragments supplies only `fragment-sync` + the seam. Opinion flows *down* into fragments via `Config` + `SyncHook`; it never flows *up* into the core.

**Break-coordination rule:** any change to the four signatures above, or to the stdout/hook-parity invariants, ripples to pagekit's 112 tests — flag and coordinate before merging, never break unilaterally.

## Net

No refactor warranted. The mechanism/opinion line is already drawn and load-bearing. This doc names it in the suite's vocabulary so the seam stays explicit as the shared core forms; the single optional clarification (`Config` → engine/policy split) is parked pending a consumer that pulls for it.
