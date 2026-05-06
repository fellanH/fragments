# fragments — marker-region sync for any text format

## What fragments is

A single Rust binary that syncs marker-region content across files. The primitive: marked regions in target files are kept identical to source files in `fragments/`. Format-agnostic — works on any text file with comment-pair syntax.

```html
<!-- fragment:head -->
...content replaced on sync...
<!-- /fragment:head -->
```

```css
/* fragment:reset */
...
/* /fragment:reset */
```

```yaml
# fragment:auth-config
...
# /fragment:auth-config
```

Every file is valid in its native format at all times — before sync, after sync, mid-edit. No template state, no placeholder syntax, no source-vs-output distinction.

## Why it exists

Bulk content management without templates. An agent (or human) edits one file in `fragments/`, runs `fragments sync`, and the change propagates across every file with the matching marker pair. No build step, no intermediate format, no framework.

The original motivating use case was vanilla HTML websites — managing nav links, shared headers, pricing across many pages without reaching for a JS framework. That use case is now best served by [`pagekit`](../pagekit), which composes fragments core and adds HTML-specific helpers (page scaffolding, DOM-aware extraction). Fragments itself stays general — useful for any text format with comment-pair syntax.

## Library API: SyncHook

Library consumers (notably `pagekit`) can register transform hooks that mutate fragment content per target file before insertion. The fragment file on disk stays canonical; the transform applies only to the copy that lands in the target's marker region.

```rust
use fragments::{Config, SyncHook, sync_all_with};

struct DepthRelativizer;
impl SyncHook for DepthRelativizer {
    fn transform(&self, _name: &str, content: &str, target: &Path, root: &Path) -> Result<String> {
        let depth = depth_of(target, root);
        Ok(rewrite_relative_hrefs(content, depth))
    }
}

let hooks: Vec<Box<dyn SyncHook>> = vec![Box::new(DepthRelativizer)];
fragments::sync_all_with(&root, &config, &hooks)?;
```

Hooks chain sequentially. The first hook receives the canonical fragment content; each subsequent hook receives the prior hook's output. Errors propagate via `?`.

For consistency, consumers calling `sync_all_with(hooks)` MUST also call `check_all_with(hooks)` — otherwise CI staleness reports will be wrong (check would compare against unhooked content while sync writes hooked content).

## Sibling: pagekit

`pagekit` is the opinionated layer for vanilla HTML site management. It depends on `fragments` for the sync primitive and adds:

- `init` — scaffold new HTML pages with semantic marker placement
- `extract` — detect shared DOM blocks via CSS selectors and extract them
- HTML-aware health checks (link integrity, framework-export anomalies)
- Recommended config defaults for static-site conventions

Use `fragments` if your need is text sync across any format. Use `pagekit` if you're managing a vanilla HTML site.

> **Stage 1 note (2026-05-06):** `init` and `extract` currently live in fragments. Stage 2 of the fork moves them into pagekit; Stage 1 is documentation reframing only.

## Agent-first design

The tool is designed so that AI agents can manage large static websites with minimal context and maximum leverage.

### What an agent needs to know

An agent working on a site managed by this tool only needs to understand:

1. **Pages are `.html` files at the root.** One file = one route. `ls *.html` shows the sitemap.
2. **Shared markup lives in `fragments/`.** Edit `fragments/body-open.html` to change the nav across all pages. Run `fragments sync`.
3. **Every file is always valid HTML.** No templates, no placeholders, no build output. What's on disk is what renders.

Three things to know. One command to run.

No component tree to trace. No import graph to resolve. No build cache to invalidate. No source-vs-output distinction. The agent edits one file, runs one command, and the change propagates.

### Bulk operations become trivial

| Task | Without tool | With tool |
|------|-------------|-----------|
| Update nav link across 30 pages | Edit 30 files | Edit `fragments/body-open.html` (1 file) |
| Change price in 6 locations | Edit 6 files, hope you got them all | Edit `fragments/pricing-amount.html` (1 file) |
| Update testimonials on 3 pages | Copy-paste HTML into 3 files | Edit `fragments/testimonials.html` (1 file) |
| Swap CTA button style site-wide | Edit every page's CTA markup | Edit `fragments/cta.html` (1 file) |
| Add a new page with full chrome | Copy another page, manually sync head/nav/footer | `fragments init about.html && fragments sync` |
| Audit what's shared vs. page-specific | Read every file, diff them | `ls fragments/` — shared. Everything else is page-specific. |

### Error surface is small

An agent can break things in exactly two ways:
1. Edit a marker region by hand (overwritten on next sync — self-healing).
2. Malform an HTML comment marker (detectable: `fragments check` reports unpaired markers).

That's it. There are no unresolved variables, no missing data files, no template syntax errors. Every file is always valid HTML.

Compare this to a React app where an agent can break the build by misplacing an import, introducing a type error, creating a circular dependency, or passing the wrong prop type. The error surface with raw HTML + this tool is fundamentally smaller.

## Capabilities

The core model is **sync**: marked regions in page files are kept identical to source files in `fragments/`. Every page is always valid, self-contained HTML — before sync, during sync, after sync. There is no intermediate template state.

### Shared fragments

```html
<!-- fragment:head -->
  <link rel="stylesheet" href="css/styles.css" />
<!-- /fragment:head -->
```

Marked regions replaced with contents of `fragments/<name>.html`. This is the `#include` of HTML.

The key property: the content between markers is **real HTML that renders**. Before sync runs, the page works. After sync runs, the page works. The markers are standard HTML comments — invisible to browsers. No syntax foreign to HTML ever appears in a page file.

### Dynamic fragment discovery

Any `fragments/<name>.html` file becomes a syncable fragment. Pages opt in by including the marker pair. No hardcoded list, no configuration needed.

A price block, a CTA row, a testimonial grid, a feature comparison table — each becomes a `fragments/<name>.html` file. Pages that share it include the marker pair. An agent edits one file, runs sync, and every page updates.

One primitive, unlimited fragments, full coverage.

### Configuration

Optional `fragments.toml` at the project root:

```toml
marker_prefix = "fragment"     # prefix in <!-- PREFIX:name --> markers
fragments_dir = "_fragments"   # folder containing fragment source files
target_dir    = "."            # where pages live, relative to project root
exclude_dirs  = []             # subdirectories to skip when scanning for pages
max_depth     = 5              # max walk depth from target_dir
```

All fields are optional. Missing file = all defaults. The `_fragments` default uses an underscore prefix so static-site hosts (CF Pages, Eleventy, Jekyll, etc.) treat the folder as infrastructure and skip it during deploy. Different projects can use different conventions — old projects can set `marker_prefix = "html-sync"` for backwards compatibility, or populate `exclude_dirs` with project-specific folders (`dist`, `build`, `public`, `node_modules`, `css`, `fonts`).

#### Custom extract candidates

`fragments extract` ships with six built-in candidates (`<nav>`, `<footer>`, `<header>`, `.navbar`, `.site-header`, `.site-footer`). Sites with non-standard layouts add their own — user entries are **appended** to the built-ins, not a replacement:

```toml
[[extract.candidates]]
name = "sidebar"           # fragment basename; produces fragments/sidebar.html
selector = "aside.sidebar" # CSS selector to find the element in the parsed DOM
tag = "aside"              # HTML tag name (used to walk the raw source)
```

All three fields are required per entry. `tag` is needed because scraper normalizes attributes — to insert markers into the original source, we walk same-tag spans and parse each candidate to find the byte-exact match.

#### Recommended excludes for common site conventions

Defaults cover asset/build folders that most sites have (`node_modules`, `css`, `fonts`, `_assets`). Per-site additions worth considering, depending on layout:

| Folder | Why exclude |
|---|---|
| `backups/` | Date-stamped prior crawls. If a backup page contains markers, sync will silently rewrite it against current fragments — destroying the as-of snapshot |
| `mockups/` | Design exploration drafts. Same risk as backups when copied from a marked-up live page |
| `_audit/` | One-off audit artifacts |
| `dist/`, `build/`, `public/` | Generated output of an upstream build (if any) |
| `archive/`, `_archive/` | Frozen historical pages |

The defaults stay conservative — only universal asset folders. Sites that use any of the above should add to their own `fragments.toml`:

```toml
exclude_dirs = ["node_modules", "tools", "css", "fonts", "_assets", "backups", "mockups", "_audit"]
```

## Patterns

### Shared-subset extraction (head with per-page title/description)

When a region is *partially* shared — most of it identical across pages, but a few values per-page — extract only the shared subset. Don't try to share the whole region.

**Example: HTML `<head>`.** Shared across pages: charset, viewport, font preloads, stylesheet links, OG image base URL. Per-page: `<title>`, `<meta description>`, canonical URL, page-specific OG metadata.

A flat fragment can't hold per-page values. The right shape:

```html
<!-- in every page's <head>: -->
<head>
  <title>About — SiteCo</title>                              <!-- per-page, inline -->
  <meta name="description" content="The about page...">      <!-- per-page, inline -->

  <!-- fragment:head-assets -->
  <meta charset="utf-8">                                     <!-- shared, synced -->
  <meta name="viewport" content="width=device-width">
  <link rel="stylesheet" href="/css/styles.css">
  <link rel="preload" href="/fonts/inter.woff2" as="font">
  <!-- /fragment:head-assets -->
</head>
```

Edit `fragments/head-assets.html` once; every page's shared head subset updates. Per-page values stay inline, hand-edited where they belong.

This pattern resolves the "fragments can't do variables" friction without breaking the file-is-truth invariant. Apply it whenever a region has the shape `[mostly-shared] + [a few per-page values]`.

## Considered and deferred

### Partials (one-shot includes) — deferred

```html
<!-- Considered: -->
<!-- include:cta-row -->
```

Would be replaced inline with contents of `partials/cta-row.html`. Unlike shared fragments, the marker disappears after insertion — the partial content becomes page-specific HTML.

**Why deferred:** Partials introduce a second mechanism alongside sync, and the "insert then customize" pattern creates problems for agents:
- After insertion, the agent can't tell which HTML came from a partial vs. was written by hand.
- If the partial source is updated and re-inserted, page-specific modifications are lost.
- The agent now has to reason about two different systems (sync vs. include) instead of one.

**How sync handles this instead:** If content is shared, use a sync fragment — the agent edits `fragments/<name>.html` and it propagates. If content is page-specific, the agent writes it directly in the page file (agents are good at this). There's no in-between "insert once then diverge" state to track.

### Variables — deferred

```html
<!-- Considered: -->
<p class="pricing-price">{{package.price}}</p>
```

**Why deferred:** `{{package.price}}` is not HTML. A file containing it doesn't render correctly in a browser — the user sees literal `{{package.price}}` instead of `€2,500`. This creates two classes of files: source templates (broken in browser) and compiled output (works in browser). That's exactly the source/dist split we're avoiding.

**How sync handles this instead:** If a price appears on multiple pages, put it in a fragment. The fragment contains the real price as real HTML. Every page that includes the marker pair gets the real value. The agent edits `fragments/pricing-amount.html` (which contains `€2,500`) and runs sync. Same single-edit propagation, but every file on disk is valid HTML at all times.

### Repeat blocks — deferred

```html
<!-- Considered: -->
<!-- repeat:testimonials -->
<figure class="testimonial">
  <blockquote>{{quote}}</blockquote>
</figure>
<!-- /repeat:testimonials -->
```

**Why deferred:** Depends on variables (`{{quote}}`), shares the same validity problem. A repeat template is not renderable HTML — it shows one placeholder instance instead of real content.

**How sync handles this instead:** The expanded testimonials block (with real content) lives in `fragments/testimonials.html`. An agent editing testimonials edits that fragment file directly — adding, removing, or reordering `<figure>` elements in real HTML. Then sync propagates it to every page that includes the `<!-- fragment:testimonials -->` markers. More verbose than a JSON array, but every file is always valid HTML.

### Conditionals — deferred

```html
<!-- Considered: -->
<!-- if:stripe_live -->
<a href="https://buy.stripe.com/...">Buy now</a>
<!-- else -->
<a href="pricing.html#offer">View offer</a>
<!-- /if:stripe_live -->
```

**Why deferred:** Conditional blocks mean the file on disk contains markup that won't be served. The file doesn't match what the user sees. This is the template-vs-output gap again.

**How to handle instead:** Maintain separate fragment variants if needed (`fragments/cta-live.html`, `fragments/cta-preview.html`) and swap which one is active. Or handle at the edge (Cloudflare Workers can rewrite HTML at serve time using lol_html). Environment-specific concerns belong at the serving layer, not in the source files.

### Nested fragments — deferred

```html
<!-- In fragments/nav.html: -->
<nav>
  <!-- fragment:logo -->
  <svg>...</svg>
  <!-- /fragment:logo -->
  <a href="/">Home</a>
</nav>
```

Fragment source files would reference other fragments via markers. The tool resolves nested references in memory before injecting into pages — `logo.html` content replaces the marker inside `nav.html`, then the fully-resolved `nav.html` replaces markers in pages. Source files on disk are never written to.

**Motivating case:** Large SVG logos or complex reusable blocks that appear inside other fragments. An agent extracts the SVG into `fragments/logo.html` once and never rewrites it — the composition happens automatically at sync time.

**Why deferred:**

- **Cycle detection required.** If `a.html` references `b` and `b.html` references `a`, resolution loops. Needs topological sort or recursion depth cap.
- **Source/output divergence.** `fragments/nav.html` on disk would contain markers, but pages receive the resolved version. This is a softer form of the template-vs-output gap the tool avoids. Fragment sources become a kind of intermediate format.
- **Debugging complexity.** Today, wrong content in a page traces to one source file. With nesting, you trace through composition layers.
- **Ordering sensitivity.** Current sync iterates alphabetically. Nesting requires dependency-aware resolution order.

**If reconsidered:** In-memory resolution (sources untouched on disk) is the conservative path. The implementation is small — resolve fragment content through `apply_fragments` before using it as replacement content — but the model change is meaningful.

### Extract command with auto-wrap — deferred

```bash
fragments extract nav --from index.html
```

Would create `fragments/nav.html` from a block in an existing page, then scan all HTML files for exact matches of that block and wrap them with marker pairs automatically. Turns a manually-duplicated block into a managed fragment in one command.

**Motivating case:** An agent has already built a 30-page site with the same nav copy-pasted into every page. Retroactively extracting it into a fragment currently requires editing every file by hand to add markers. The extract command would automate this.

**Why deferred:**

- **Exact matching is brittle.** A trailing newline, different indentation, or extra whitespace in any page copy causes a silent miss. The tool would report "wrapped 18 of 30 pages" and the remaining 12 stay unmanaged with no obvious reason why.
- **Normalization is risky.** Collapsing whitespace for fuzzy matching improves recall but opens false positives — wrapping blocks that look similar but aren't the same content.
- **Divergent copies.** Pages may have slightly different versions of the "same" block. Which becomes the canonical source? First match? Longest? Agent-specified? Every heuristic has failure modes.
- **High-stakes mutation.** The command modifies every matched file in one operation. A bug in matching logic corrupts the site. Requires a `--dry-run` mode at minimum.

**If reconsidered:** The safest version is explicit and conservative: agent specifies the source file and the exact block (by line range or content), tool creates the fragment, then only wraps pages where a byte-exact match is found. Pages that don't match are reported but untouched. The current explicit marker model is less convenient but more reliable — which is what agents need.

### When these might return

If the sync-only model proves insufficient for a real use case that can't be solved with more granular fragments, these capabilities can be reconsidered. The bar is: does the benefit outweigh the cost of added complexity and new failure modes? For now, the answer is no — sync with explicit markers gets us very far.

## What this is NOT

- **Not a template engine.** No variables, no loops, no expressions, no placeholder syntax. Every file is valid in its native format at all times. See "Considered and deferred" for why.
- **Not a build system.** It transforms existing files in place. You own the file tree.
- **Not format-aware.** Files are treated as text streams with marker pairs. For format-specific operations (HTML scaffolding, DOM-aware extraction, link integrity), see [`pagekit`](../pagekit).
- **Not a human-first DX tool that agents happen to use.** The primary user is an AI agent. The design choices optimize for agent legibility, predictable file I/O, small error surfaces, and one-command propagation. Humans benefit from the same properties, but the design is agent-first.

## Architecture

```
project-root/
  index.html            ← pages (you edit these)
  pricing.html
  about.html
  _fragments/           ← shared regions (synced into marker pairs);
    head.html             underscore-prefixed so deploy hosts skip it
    body-open.html
    body-close.html
    cta.html            ← any shared content block
    pricing-amount.html
    testimonials.html
  fragments.toml        ← optional config
  css/styles.css
  fonts/
  favicon.svg
```

The binary scans `*.html` at root and replaces every marker region with the corresponding `fragments/<name>.html` contents. One pass, one mechanism. Files are only written when content changes (byte comparison).

## Modes

| Command | Behavior |
|---------|----------|
| `fragments sync` | One-shot: process all pages |
| `fragments watch` | Sync, then watch `fragments/` for changes |
| `fragments check` | Dry-run: exit 1 if any page is stale or has unpaired markers (CI/pre-commit) |
| `fragments init <file>` | Create new page with marker pairs for all fragments |
| `fragments extract` | Detect duplicated DOM blocks across pages, extract to fragments/, insert markers |
| `fragments list` | List every fragment and how many pages reference it |
| `fragments config` | Print the effective config (defaults merged with `fragments.toml`) |
| `fragments doctor` | Health check: surface orphan fragments, orphan markers, unpaired markers; exit 1 on issues |

## Design principles

1. **The file is the truth.** After sync, every `.html` file is a valid, self-contained HTML document. No runtime resolution. What's on disk is what gets served. An agent reads the file and sees what the user sees.
2. **The folder is the site.** `ls` shows you every route. Double-click to preview. Upload to deploy. No build artifacts, no `dist/` directory. An agent runs `ls *.html` and knows the sitemap.
3. **Output = input.** The tool writes the same format it reads. You can hand-edit any output file and it remains valid input. An agent can edit output files without understanding the tool.
4. **Preserve authorship.** Whitespace, comments, attribute order — all preserved. Diffs are minimal and reviewable.
5. **Single binary, zero dependencies.** No `node_modules`, no package manager, no version matrix. Copy the binary, it works. An agent doesn't need to resolve dependency conflicts or manage lockfiles.
6. **One edit, one command, full propagation.** The agent edits one file in `fragments/`, runs `fragments sync`, and the change appears across all affected pages. No manual coordination.
7. **Machine-verifiable correctness.** `fragments check` exits non-zero if anything is stale, unresolved, or malformed. An agent can run it after every edit to confirm the site is consistent — no visual inspection needed.

## Agent workflow

A typical agent session managing a 30-page marketing site:

```
1. Agent reads task: "Update pricing from €2,500 to €2,900 and add a new testimonial"

2. Agent runs: ls *.html → sees all 30 pages (sitemap)
   Agent runs: ls fragments/ → sees all shared fragments

3. Agent edits fragments/pricing-amount.html: changes €2,500 to €2,900
   Agent edits fragments/testimonials.html: adds a new <figure> block

4. Agent runs: fragments sync
   → 12 pages updated (those with pricing-amount or testimonials markers)
   → 18 pages unchanged

5. Agent runs: fragments check → exit 0 (all consistent)

6. Agent commits and deploys.
```

Total files the agent touched: 2 (both in `fragments/`).
Total files that changed on disk: 12.
Zero chance of missing a page or introducing inconsistency.
Every file on disk — source and output — is valid HTML at every step.

## Implementation status

| Phase | What | Status |
|-------|------|--------|
| 0 | Shared fragments (3 fixed names) | Done |
| 1 | Dynamic fragment discovery (`fragments/<any>.html`) | Done |
| 1b | Manifest config (`fragments.toml`) | Done |
| 1c | Init command (`fragments init <file>`) | Done |
| 2 | Rename from `html-sync` to `fragments` | Done |
| 3 | Extract command (duplicate-detection variant: `fragments extract`) | Done |
| — | Extend beyond HTML to other text formats | Future |
| — | Nested fragments (composition within fragment sources) | Deferred |
| — | Extract command with auto-wrap (exact-match all pages from one source) | Deferred |
| — | Reverse sync (page → source, `fragments pull`) | Deferred |

Partials, variables, repeats, conditionals, nested fragments, extract, and reverse sync are documented in "Considered and deferred" above. They can be reconsidered if the sync-only model proves insufficient for a concrete use case.
