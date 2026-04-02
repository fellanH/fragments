# fragments — shared content sync for static files

## Problem

HTML has no `#include`, no way to share markup across files. Every modern web framework exists primarily to paper over this gap. But frameworks introduce build steps, runtime JS, intermediate formats (JSX, Astro, Svelte), `node_modules`, version conflicts, and an abstraction layer between what you write and what gets served.

For static sites — marketing pages, documentation, portfolios — this is a terrible trade. You don't need a virtual DOM or hydration. You need shared headers, reusable sections, and a way to update a nav link in one place and have it propagate across thirty pages.

**For AI agents, frameworks are an even worse trade.** An agent managing a 50-page marketing site needs to:
- Update a nav link across every page (bulk mutation)
- Change a price that appears in six places (consistency)
- Add a new testimonial to three pages at once (data-driven content)
- Swap a CTA across the site without touching page-specific content (scoped edits)
- Understand what it's looking at without parsing JSX/TSX/Astro/MDX (legibility)

With React, an agent must understand component trees, props, context, imports, build pipelines, and framework conventions. With plain HTML, an agent reads a file and sees exactly what the browser sees — but it has no way to make a change in one place and have it propagate.

This tool gives agents (and humans) both: **the legibility of raw HTML and the leverage of shared abstractions.**

## Vision

A single Rust binary that closes HTML's gaps without changing the format. The input is `.html` files. The output is `.html` files. There is no intermediate representation, no build artifact, no framework. The tool **syncs shared content across files** — you define a fragment once, mark where it belongs, and the tool keeps every page consistent.

Every file is valid, self-contained HTML at all times — before sync, after sync, mid-edit. There is no template state, no placeholder syntax, no source-vs-output distinction. The output format is the input format. You can stop using the tool at any time and keep your files.

The tool may extend beyond HTML in the future — the sync primitive works on any text file that supports comment markers.

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
fragments_dir = "fragments"    # folder containing fragment source files
```

Both fields are optional. Missing file = all defaults. Different projects can use different conventions — old projects can set `marker_prefix = "html-sync"` for backwards compatibility.

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

### When these might return

If the sync-only model proves insufficient for a real use case that can't be solved with more granular fragments, these capabilities can be reconsidered. The bar is: does the benefit of a JSON-driven data layer outweigh the cost of source files that don't render standalone? For now, the answer is no — sync gets us very far.

## What this is NOT

- **Not a framework.** No runtime JS, no virtual DOM, no hydration, no client-side routing.
- **Not a template engine.** No variables, no loops, no expressions, no placeholder syntax. Every file is valid HTML at all times. See "Considered and deferred" for why.
- **Not a CMS.** The tool doesn't provide a GUI, a database, or an API. You edit HTML files directly.
- **Not a site generator.** It doesn't create files from a schema. It transforms existing files in place. You own the file tree.
- **Not a human-first DX tool that agents happen to use.** The primary user is an AI agent. The design choices optimize for agent legibility, predictable file I/O, small error surfaces, and one-command propagation. Humans benefit from the same properties, but the design is agent-first.

## Architecture

```
project-root/
  index.html            ← pages (you edit these)
  pricing.html
  about.html
  fragments/
    head.html           ← shared regions (synced into marker pairs)
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
| `fragments check` | Dry-run: exit 1 if any page is stale (CI/pre-commit) |
| `fragments init <file>` | Create new page with marker pairs for all fragments |

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
| — | Extend beyond HTML to other text formats | Future |

Partials, variables, repeats, and conditionals are documented in "Considered and deferred" above. They can be reconsidered if the sync-only model proves insufficient for a concrete use case.
