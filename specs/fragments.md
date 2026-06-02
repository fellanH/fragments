# fragments — marker-region sync for any text format

## What fragments is

A single Rust binary that syncs marker-region content across files. The
primitive: marked regions in target files are kept identical to source files in
`_fragments/`. Format-agnostic — markers are ordinary comments in the *target
file's own format*, so the same fragment syncs into any text format that has a
comment syntax.

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

Every file is valid in its native format at all times — before sync, after
sync, mid-edit. No template state, no placeholder syntax, no source-vs-output
distinction.

## Why it exists

Bulk content management without templates. An agent (or human) edits one file in
`_fragments/`, runs `fragments sync`, and the change propagates across every
file with the matching marker pair. No build step, no intermediate format, no
framework.

The original motivating use case was vanilla HTML websites — managing nav links,
shared headers, pricing across many pages without reaching for a JS framework.
That use case is now best served by [`pagekit`](../pagekit), which composes
fragments core and adds HTML-specific helpers (page scaffolding, DOM-aware
extraction, link integrity). Fragments itself stays general — useful for any
text format with comment-pair syntax.

## Comment syntax per format

Markers are built from the comment syntax of the file they live in. fragments
resolves that syntax from the file's extension (or, for extensionless files like
`Makefile`, its name) via a built-in table:

| Comment style | Extensions (built-in) |
| --- | --- |
| `<!-- … -->` | html, htm, xhtml, xml, svg, vue, svelte, md, markdown |
| `/* … */` | css, scss, less, js, mjs, cjs, jsx, ts, tsx, c, cc, cpp, h, hpp, java, go, rs, swift, kt, php, scala, dart |
| `# …` (line) | yaml, yml, toml, sh, bash, zsh, fish, py, rb, pl, r, conf, cfg, ini, env, Dockerfile, Makefile, .gitignore |
| `-- …` (line) | sql, lua, hs, elm |

Block-comment markers (`<!-- fragment:x -->`) carry their closing delimiter;
line-comment markers (`# fragment:x`) run to end-of-line. Formats not in the
table are invisible to fragments unless declared in config (below). Markdown
maps to HTML comments because that is what renders invisibly in Markdown.

A fragment's **name** is its file stem (`nav.html` → `nav`); the fragment file's
own extension is irrelevant to matching. The same fragment named `notice` syncs
into `index.html` (`<!-- fragment:notice -->`), `style.css`
(`/* fragment:notice */`), and `deploy.sh` (`# fragment:notice`) at once — each
target staying valid in its native format. Two source files that resolve to the
same name (e.g. `header.html` + `header.css`) are an error: the name is
ambiguous and sync refuses rather than silently picking one.

## Library API: SyncHook

Library consumers (notably `pagekit`) can register transform hooks that mutate
fragment content per target file before insertion. The fragment file on disk
stays canonical; the transform applies only to the copy that lands in the
target's marker region.

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

Hooks chain sequentially. The first hook receives the canonical fragment
content; each subsequent hook receives the prior hook's output. Errors propagate
via `?`.

For consistency, consumers calling `sync_all_with(hooks)` MUST also call
`check_all_with(hooks)` and `watch::run_with(hooks)` — otherwise CI staleness
reports or reactive resyncs will produce different output than initial sync.

The stable high-level surface is `sync_all` / `sync_all_with`, `check_all` /
`check_all_with`, `watch::run` / `watch::run_with`, `Config`, and the `SyncHook`
trait. `CommentSyntax` and `referenced_fragment_names` are also exported for
consumers that need to reason about markers directly.

## Sibling: pagekit

`pagekit` is the opinionated layer for vanilla HTML site management. It depends
on `fragments` for the sync primitive and adds:

- `init` — scaffold new HTML pages with semantic marker placement
- `extract` — detect shared DOM blocks via CSS selectors and extract them
- HTML-aware health checks (link integrity, framework-export anomalies)
- Recommended config defaults for static-site conventions (the `css`, `fonts`,
  `_assets`, `dist`, `node_modules` exclude set, etc.)

Use `fragments` if your need is text sync across any format. Use `pagekit` if
you're managing a vanilla HTML site. `init` and `extract` moved from fragments
to pagekit when the fork shipped (Stage 2); fragments core no longer carries any
HTML-specific command.

## Agent-first design

The tool is designed so that AI agents can manage large content trees with
minimal context and maximum leverage. The examples below use a static HTML site
(the canonical case) but the model applies to any format.

### What an agent needs to know

An agent working on a tree managed by this tool only needs to understand:

1. **Target files live where you point it.** One file = one route/document.
   `ls` shows the tree.
2. **Shared content lives in `_fragments/`.** Edit `_fragments/nav.html` to
   change the nav everywhere. Run `fragments sync`.
3. **Every file is always valid in its own format.** No templates, no
   placeholders, no build output. What's on disk is what renders.

Three things to know. One command to run.

No component tree to trace. No import graph to resolve. No build cache to
invalidate. No source-vs-output distinction. The agent edits one file, runs one
command, and the change propagates.

### Bulk operations become trivial

| Task | Without tool | With tool |
|------|-------------|-----------|
| Update nav link across 30 pages | Edit 30 files | Edit `_fragments/nav.html` (1 file) |
| Change price in 6 locations | Edit 6 files, hope you got them all | Edit `_fragments/pricing-amount.html` (1 file) |
| Sync a license header across all source files | Edit every file | Edit `_fragments/license.txt`, mark each file once |
| Keep a shared CI snippet identical in 4 workflow YAMLs | Copy-paste, drift | Edit `_fragments/ci-steps.yaml` (1 file) |
| Audit what's shared vs. file-specific | Read every file, diff them | `ls _fragments/` — shared. Everything else is local. |

### Error surface is small

An agent can break things in exactly two ways:
1. Edit a marker region by hand (overwritten on next sync — self-healing).
2. Malform a comment marker (detectable: `fragments check` reports unpaired or
   duplicate markers).

That's it. There are no unresolved variables, no missing data files, no template
syntax errors. Every file is always valid in its native format.

## Capabilities

The core model is **sync**: marked regions in target files are kept identical to
source files in `_fragments/`. Every target is always valid, self-contained
content — before sync, during sync, after sync. There is no intermediate
template state.

### Shared fragments

```html
<!-- fragment:head -->
  <link rel="stylesheet" href="css/styles.css" />
<!-- /fragment:head -->
```

Marked regions replaced with contents of `_fragments/<name>`. This is the
`#include` of plain files. The content between markers is **real content that
renders/runs**. Before sync, the file works. After sync, the file works. The
markers are standard comments in the file's format — invisible to the
renderer/interpreter.

### Dynamic fragment discovery

Any non-hidden file in `_fragments/` becomes a syncable fragment, named by its
stem. Targets opt in by including the marker pair. No hardcoded list, no
per-fragment configuration.

One primitive, unlimited fragments, full coverage.

### Configuration

Optional `fragments.toml` at the project root:

```toml
marker_prefix = "fragment"     # prefix in the <prefix>:name markers
fragments_dir = "_fragments"   # folder containing fragment source files
target_dir    = "."            # where target files live, relative to root
exclude_dirs  = []             # subdirectories to skip when scanning
max_depth     = 5              # max walk depth from target_dir

# Extend or override the built-in comment-syntax table. Key = file extension
# (or file name for extensionless files). Value = [open, close]; an empty
# close means a line comment terminated by end-of-line.
[syntax]
njk = ["{#", "#}"]   # Nunjucks block comment
```

All fields are optional; a missing file means all defaults. Notable defaults:

- **`exclude_dirs` is empty** — config over convention. The core ships no
  built-in excludes; each consumer declares what it wants skipped. Format-shaped
  default sets (`css`, `fonts`, `_assets`, `dist`, `build`, `node_modules`, …)
  belong in a consumer's config layer (e.g. pagekit) or a per-project
  `fragments.toml`, not the primitive.
- **`fragments_dir = "_fragments"`** — the underscore prefix makes static-site
  hosts (CF Pages, Eleventy, Jekyll) treat the folder as infrastructure and skip
  it during deploy.

Old projects can set `marker_prefix = "html-sync"` for backwards compatibility.

## Patterns

### Shared-subset extraction (head with per-page title/description)

When a region is *partially* shared — most of it identical across files, but a
few values per-file — extract only the shared subset. Don't try to share the
whole region.

**Example: HTML `<head>`.** Shared across pages: charset, viewport, font
preloads, stylesheet links. Per-page: `<title>`, `<meta description>`, canonical
URL.

```html
<!-- in every page's <head>: -->
<head>
  <title>About — SiteCo</title>                              <!-- per-page, inline -->
  <meta name="description" content="The about page...">      <!-- per-page, inline -->

  <!-- fragment:head-assets -->
  <meta charset="utf-8">                                     <!-- shared, synced -->
  <meta name="viewport" content="width=device-width">
  <link rel="stylesheet" href="/css/styles.css">
  <!-- /fragment:head-assets -->
</head>
```

Edit `_fragments/head-assets.html` once; every page's shared head subset
updates. Per-page values stay inline, hand-edited where they belong.

This pattern resolves the "fragments can't do variables" friction without
breaking the file-is-truth invariant. Apply it whenever a region has the shape
`[mostly-shared] + [a few per-file values]`.

## Considered and deferred

The reasoning below is format-neutral; the HTML examples are illustrative.

### Partials (one-shot includes) — deferred

```html
<!-- Considered: -->
<!-- include:cta-row -->
```

Would be replaced inline with contents of `partials/cta-row`. Unlike shared
fragments, the marker disappears after insertion — the partial content becomes
file-specific.

**Why deferred:** Partials introduce a second mechanism alongside sync, and the
"insert then customize" pattern creates problems for agents:
- After insertion, the agent can't tell which content came from a partial vs.
  was written by hand.
- If the partial source is updated and re-inserted, file-specific modifications
  are lost.
- The agent now has to reason about two different systems (sync vs. include)
  instead of one.

**How sync handles this instead:** If content is shared, use a sync fragment. If
content is file-specific, the agent writes it directly. There's no in-between
"insert once then diverge" state to track.

### Variables — deferred

```html
<!-- Considered: -->
<p class="pricing-price">{{package.price}}</p>
```

**Why deferred:** `{{package.price}}` is not valid in the host format. A file
containing it doesn't render correctly — the user sees literal
`{{package.price}}`. This creates two classes of files: source templates (broken)
and compiled output (works). That's exactly the source/dist split we're avoiding.

**How sync handles this instead:** If a value appears in multiple files, put it
in a fragment containing the real value. Every file that includes the marker
gets the real value. Same single-edit propagation, but every file on disk is
always valid.

### Repeat blocks — deferred

**Why deferred:** Depends on variables, shares the same validity problem. A
repeat template is not renderable content — it shows one placeholder instance.

**How sync handles this instead:** The expanded block (with real content) lives
in a fragment. An agent edits that file directly — adding, removing, reordering
real elements — then sync propagates it.

### Conditionals — deferred

**Why deferred:** Conditional blocks mean the file on disk contains content that
won't be served. The file doesn't match what the user sees. The
template-vs-output gap again.

**How to handle instead:** Maintain separate fragment variants
(`_fragments/cta-live.html`, `_fragments/cta-preview.html`) and swap which is
active. Or handle at the edge (e.g. Cloudflare Workers rewriting at serve time).
Environment-specific concerns belong at the serving layer, not the source files.

### Nested fragments — deferred

Fragment source files would reference other fragments via markers; the tool
resolves nested references in memory before injecting into targets. Source files
on disk are never written to.

**Why deferred:**
- **Cycle detection required.** `a` → `b` → `a` loops; needs topological sort or
  a recursion depth cap.
- **Source/output divergence.** A fragment source on disk would contain markers
  while targets receive the resolved version — a softer form of the
  template-vs-output gap the tool avoids.
- **Debugging complexity.** Today wrong content traces to one source file; with
  nesting you trace through composition layers.
- **Ordering sensitivity.** Current sync iterates alphabetically; nesting needs
  dependency-aware order.

**If reconsidered:** In-memory resolution (sources untouched on disk) is the
conservative path. The implementation is small — resolve fragment content
through `apply_fragments` before using it as replacement content — but the model
change is meaningful.

### Reverse sync (target → source, `fragments pull`) — deferred

Would let an edit inside a target's marker region propagate back to the fragment
source. Deferred: it inverts the single-source-of-truth direction and reopens
"which copy wins" questions that the one-way model deliberately closes.

### When these might return

If the sync-only model proves insufficient for a real use case that can't be
solved with more granular fragments, these can be reconsidered. The bar is: does
the benefit outweigh the cost of added complexity and new failure modes? For now
the answer is no — sync with explicit markers gets us very far.

## What this is NOT

- **Not a template engine.** No variables, no loops, no expressions, no
  placeholder syntax. Every file is valid in its native format at all times. See
  "Considered and deferred" for why.
- **Not a build system.** It transforms existing files in place. You own the
  file tree.
- **Not a parser.** It is format-*aware* only to the extent of knowing each
  format's comment delimiters (so markers are valid comments). It does not parse
  or understand file structure. For structure-aware operations (HTML
  scaffolding, DOM extraction, link integrity), see [`pagekit`](../pagekit).
- **Not a human-first DX tool that agents happen to use.** The primary user is an
  AI agent. The design optimizes for agent legibility, predictable file I/O,
  small error surfaces, and one-command propagation. Humans benefit from the same
  properties, but the design is agent-first.

## Architecture

```
project-root/
  index.html            ← target files (you edit these)
  style.css
  deploy.sh
  _fragments/           ← shared regions (synced into marker pairs);
    head.html             underscore-prefixed so deploy hosts skip it
    notice.txt          ← any shared content block, any extension
    license.txt
  fragments.toml        ← optional config
```

The binary scans `target_dir` for files whose format has a known comment syntax,
and replaces every marker region with the corresponding `_fragments/<name>`
contents, using that target's comment delimiters. One pass, one mechanism. Files
are only written when content changes (byte comparison).

### Module map

- `syntax.rs` — `CommentSyntax`, the built-in extension→syntax table, and config
  override resolution.
- `config.rs` — `fragments.toml` schema, defaults, and `syntax_for(path)`.
- `sync.rs` — fragment loading, marker scanning/replacement, `sync_all` /
  `check_all` and their `_with(hooks)` variants, the `SyncHook` trait.
- `list.rs`, `doctor.rs` — reference mapping and health checks.
- `watch.rs` — debounced re-sync loop.

### Write durability

Sync writes targets via a direct truncate-and-write (`fs::write`), not
tempfile+rename. A SIGKILL or power loss mid-write can leave a partial file;
recovery is `fragments sync` again (idempotent). This trade keeps inode, perms,
and xattrs intact and was chosen deliberately — see `tasks/arc.md`.

## Modes

| Command | Behavior |
|---------|----------|
| `fragments sync` | One-shot: process all target files (default) |
| `fragments watch` | Sync, then watch `_fragments/` for changes |
| `fragments check` | Dry-run: exit 1 if any target is stale or has unpaired/duplicate markers (CI/pre-commit) |
| `fragments list` | List every fragment and how many files reference it |
| `fragments config` | Print the effective config (defaults merged with `fragments.toml`) |
| `fragments doctor` | Health check: orphan fragments, orphan markers, unpaired/duplicate markers; exit 1 on issues |

`check`, `list`, and `doctor` accept `--json` for machine-readable output
(agent/CI consumers); each emits a stable `kind`-tagged schema and exit codes
are unchanged. (`init` and `extract` are pagekit commands, not fragments.)

## Design principles

1. **The file is the truth.** After sync, every file is valid, self-contained
   content in its native format. No runtime resolution. What's on disk is what
   gets served.
2. **The folder is the project.** `ls` shows you everything. No build artifacts,
   no `dist/` directory.
3. **Output = input.** The tool writes the same format it reads. Hand-edit any
   output file and it remains valid input.
4. **Preserve authorship.** Whitespace, comments, attribute order — all
   preserved. Diffs are minimal and reviewable.
5. **Single binary, zero runtime dependencies.** No `node_modules`, no package
   manager, no version matrix.
6. **One edit, one command, full propagation.**
7. **Machine-verifiable correctness.** `fragments check` exits non-zero if
   anything is stale or malformed.

## Implementation status

| Phase | What | Status |
|-------|------|--------|
| 0 | Shared fragments (3 fixed names) | Done |
| 1 | Dynamic fragment discovery | Done |
| 1b | Manifest config (`fragments.toml`) | Done |
| 2 | Rename from `html-sync` to `fragments` | Done |
| 3 | Fork: HTML helpers (`init`, `extract`) moved to pagekit; fragments exposes a library | Done |
| 4 | `SyncHook` API + hookable watch (v0.6.x) | Done |
| 5 | **Format-agnostic comment syntax** + `[syntax]` config (v0.7.0) | Done |
| — | Nested fragments (composition within fragment sources) | Deferred |
| — | Partials / variables / repeats / conditionals | Deferred |
| — | Reverse sync (target → source, `fragments pull`) | Deferred |

Deferred items are documented in "Considered and deferred" above. They can be
reconsidered if the sync-only model proves insufficient for a concrete use case.
