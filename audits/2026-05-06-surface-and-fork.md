# Surface audit + fork rationale — 2026-05-06

Point-in-time audit that drove the fragments → pagekit fork. Written after the work landed, so it's a record, not a plan. Captures the surface, the friction, the strategic call, and the resulting architecture.

## Context

After Sprint 1 (the original `fragments` v0.5.0 audit on this date earlier in the day), two real consumers existed: `felixhellstrom.com` (integrated, 27 pages, 7 fragments) and `ettsmart.se` (in progress, hotel/property site). Both were vanilla HTML websites. The tool had been positioned in spec as "format-agnostic primitive that may extend beyond HTML in the future" — but the user demand was uniformly HTML-shaped, and the codebase had been accumulating HTML-specific features (init template, extract DOM heuristics, default exclude_dirs of `css`/`fonts`/`_assets`) that fit awkwardly under the "general primitive" framing.

The audit asked: do we keep growing one tool, or split?

## Section 1 — surface as audited

### CLI commands at audit time (8 in `fragments` binary)

| Command | Purpose | Exit codes |
|---|---|---|
| `sync` | Replace marker-region content in pages with `fragments/<name>.<ext>` source | 0 always |
| `watch` | Sync once, then watch `fragments/` for changes (80ms debounce) | 0 graceful |
| `check` | Dry-run: report stale + unpaired + duplicate markers | 0 clean / 1 issues |
| `init` | Scaffold new HTML page with marker pairs (HTML-specific) | 0 created / nonzero on collision |
| `extract` | Detect shared DOM blocks, write fragment files, insert markers (HTML-specific) | 0 always |
| `list` | Print fragment + page reference count | 0 always |
| `config` | Print effective config as TOML | 0 always |
| `doctor` | Health check: orphan fragments, orphan markers, malformed markers | 0 clean / 1 issues |

Plus `--version` and `--help` global flags.

### Config fields (`fragments.toml`)

| Field | Type | Default at audit | Notes |
|---|---|---|---|
| `marker_prefix` | `String` | `"fragment"` | `"html-sync"` for legacy projects |
| `fragments_dir` | `String` | `"fragments"` | felixhellstrom overrode to `"_fragments"` |
| `target_dir` | `String` | `"."` | Where pages live |
| `exclude_dirs` | `Vec<String>` | `["node_modules", "tools", "css", "fonts", "_assets"]` | Mostly website conventions |
| `max_depth` | `usize` | `5` | Walk depth |
| `[[extract.candidates]]` | List | `[]` | HTML-specific; appends to 6 built-ins |

### Marker syntax

```html
<!-- fragment:NAME -->
...content replaced on sync...
<!-- /fragment:NAME -->
```

Constraints:
- `NAME` matches `[a-zA-Z0-9_-]+`
- Open/close paired by name (stack-style)
- First pair wins on duplicate-name pairs in same file (silent footgun pre-Sprint 2)

### What was explicitly out of scope per spec

Variables, partials, conditionals, repeats, nested fragments, reverse sync, non-HTML extensions, auto-wrap variant of extract. All deferred on the file-must-be-valid-at-all-times invariant.

## Section 2 — friction the audit surfaced

### From felixhellstrom.com (integrated)

1. **Head fragment was structurally unwirable as flat content.** Pages need per-page `<title>` and `<meta description>`; flat fragments can't hold per-page values. Agent deleted `head.html` rather than refactor — correctly recognized the deferred-feature class without asking for templates.
2. **8 orphan fragments** (7 Webflow scaffolding stubs + the head fragment). All deleted.
3. **15 of 35 scanned pages were non-live** (`backups/`, `mockups/`, `_audit/`). None had markers, so sync was a no-op against them — but a future copy-marked-page-into-mockup workflow would silently rewrite the snapshot. Not a current bug, a real future class.

### From ettsmart.se (mid-build)

4. **"Navbar is a fragment so we need variants on top of that."** The seat surfaced this as a future-architecture issue, suggesting a relay to chad-fragments. Pattern was already implicitly solved by felixhellstrom (5 nav fragments — `nav`, `nav-blog`, `nav-blog-index`, `nav-l1`, `nav-project`) but undocumented. Granular-fragments pattern is the answer; doc was missing.

### From the surface itself

5. **Hardcoded scan exclusions** lived in code, not config. `tools/`, `node_modules/`, `css/`, `fonts/` baked in.
6. **Hardcoded `max_depth(5)`.** Sites with deeper trees silently invisible.
7. **6 hardcoded extract candidates.** Sites using `.brand-bar`, `.menu-primary`, etc. got nothing.
8. **No discoverability commands.** No way to ask "which pages reference fragment X?" or "what's the effective config?" or "are there orphan fragments?" without grepping by hand.
9. **`init` template was opinionated HTML.** Couldn't fit a non-HTML primitive.
10. **`extract` algorithm had a source-vs-DOM reconciliation hack** (`find_first_tag_span` + `find_matching_tag_span`). Worked, but every same-tag-siblings case risked wrapping the wrong element.

## Section 3 — the strategic call

The friction told two stories at once:

1. The tool needed real config-driven flexibility (#5–8 are config gaps in the primitive).
2. The HTML-specific accumulation (#9, #10, plus init's DOCTYPE template, plus default exclude_dirs leaning into website conventions) was conceptual debt under the "general primitive" framing.

Two valid paths:

| Path | Pro | Con |
|---|---|---|
| **Embrace the niche** — drop "may extend beyond HTML" framing, lean fully into vanilla HTML site management | Simpler, one tool, matches actual usage | Closes door on general text sync forever; reversible only via lift-rewrite |
| **Fork into two workspaces** — `fragments` stays general, new `pagekit` consumes it for HTML | Each tool sharp; future general-text consumers welcome | Two surfaces, two repos, migration cost for existing consumers |

Felix picked fork on the rationale that "the fragments naming is more suitable for a general unopinionated tool" + "we might benefit from a separate workspace for the website-specific tool." The strategic value of keeping the primitive available for non-HTML use cases (CSS/JS/MD/YAML sync) outweighed the cost of two surfaces — especially given the path-dep model means fragments fixes propagate to pagekit transparently.

## Section 4 — stability assessment per surface element (at audit time)

| Surface | Stability | Comment |
|---|---|---|
| 8 commands | Stable | No churn, all tested |
| `marker_prefix`, `fragments_dir`, `target_dir` | Stable | Original three fields, no changes |
| `exclude_dirs`, `max_depth` | Probably stable | Just shipped (Sprint 2 P0) |
| `[[extract.candidates]]` | Probably stable | Just shipped (Sprint 2 P1) |
| Default `fragments_dir = "fragments"` | **Open question** | felixhellstrom (only consumer at time) overrode to `_fragments` |
| Internal Rust API | Unstable | No `lib.rs` yet at audit time |

The "open question" was the trigger for keeping fragments small + format-agnostic — if the spec default is wrong for the dominant consumer, that's signal the spec was speculating about non-existent users.

## Section 5 — outcome (commits + state)

Sprint 2 P0/P1/P2 (config-hardening + discoverability) shipped before the fork:
- `3421cdc` — `exclude_dirs` and `max_depth` configurable
- `2dc1a2b` — `[[extract.candidates]]` user-defined
- `ff00e4c` — `fragments list`, `fragments config`, `fragments doctor`
- `2ecaa7e` — duplicate marker pair detection in `check`/`doctor`

Sprint 3 (the fork itself), three stages:

| Stage | Commit (fragments) | Commit (pagekit) | What landed |
|---|---|---|---|
| 1 — framing | `bf43985` | `e24c2e6` | Spec rewritten as format-agnostic primitive; pagekit workspace scaffolded with AGENTS.md, spec, arc, workspace.yaml; default `exclude_dirs` cleared |
| 2 — code split | `0be192e` | `c77d8cf` | `fragments/src/lib.rs` exposes public API; `init.rs`, `extract.rs`, `ExtractConfig` moved to pagekit; pagekit binary builds with all 8 commands (delegating sync/check/etc. to fragments lib, owning init/extract); 13 tests moved with the code |
| 3 — lol_html | n/a | `ba0cfcf` | pagekit's extract rewritten on hybrid scraper-detect + lol_html-rewrite; sibling-index bridge; `find_first_tag_span` and `find_matching_tag_span` deleted; the source-vs-DOM bug class eliminated |

Test counts:
- Pre-fork: 40 tests in fragments
- Post-fork: 27 tests in fragments + 13 tests in pagekit = 40 total. No tests lost.

Both binaries verified against felixhellstrom.com — `pagekit check` and `fragments check` produce identical output reading the same `fragments.toml` (the flatten makes pagekit's config a transparent superset).

## Architecture as it stands

```
   ┌─────────────────────────────────────────────┐
   │  pagekit (binary)                           │
   │  HTML-specific layer                        │
   │  init, extract (lol_html-based source       │
   │  rewrite), 8 commands total                 │
   └────────────────┬────────────────────────────┘
                    │ depends on (Rust path dep)
                    ▼
   ┌─────────────────────────────────────────────┐
   │  fragments (lib + binary)                   │
   │  format-agnostic primitive                  │
   │  sync, watch, check, list, doctor           │
   │  6 commands in standalone binary            │
   └─────────────────────────────────────────────┘
```

Users see one config file, one binary (pagekit for HTML sites, fragments for non-HTML text sync), one CLI. Underneath, two concerns cleanly separated.

## Open thread at audit close

The default `fragments_dir` was still `"fragments"` per the spec at the time of this audit. felixhellstrom (n=1 consumer) had explicitly overridden to `_fragments`. The decision rule was "wait for n=2" — but Felix subsequently confirmed (post-audit) that all his sites use the underscore convention because the underscore signals to deploy tools (CF Pages and similar) that the directory is not part of the deployable site. Default flipped to `_fragments` accordingly; this audit is the record of the prior state.

## Lessons worth keeping

- **n=2 is the right threshold for spec defaults.** With one consumer you're guessing; with two consumers and concordant behavior, you have a signal. The `fragments_dir` open question was honest about this.
- **Friction surfaces specialization, not generalization.** Every single piece of friction the audit found was HTML-specific. The "general primitive" framing was speculative; specialization was already the truth on disk.
- **Path-dep crates compose cleanly.** Rust's `path = "../fragments"` + `serde::flatten` made the two-workspace split mechanically simple. Each repo can be released independently; cross-cutting fixes propagate transparently via build.
- **Source-vs-DOM reconciliation is a bug class, not a workaround.** The lol_html migration eliminated a class of failures by choosing a primitive that operates on source bytes directly. When workarounds proliferate around a primitive, the primitive is wrong.
- **The agent caught the head-fragment limitation correctly.** No relay needed, no spec consultation needed, no template syntax requested. The spec invariants were absorbed into agent behavior. This is what good docs look like — the limits are visible enough that consumers don't fight them.
