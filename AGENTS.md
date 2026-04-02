# html-sync

A single Rust binary that syncs shared HTML fragments across static site pages. One primitive: marked regions in page files are kept identical to source files in `inject/`. Every file is valid, self-contained HTML at all times.

**Vision spec:** `specs/html-compiler.md` — the full product vision, agent-first design, and implementation path.

## Stack

- **Language:** Rust (2021 edition)
- **Dependencies:** clap 4, walkdir 2, notify 7, anyhow 1
- **Binary:** `html-sync` (will be renamed to `kaizen` in a future phase)

## Commands

```bash
cargo build --release          # build
cargo install --path .         # install to ~/.cargo/bin/
cargo test                     # run tests (when added)
```

## Usage

```bash
html-sync <site-root> sync     # one-shot sync (default)
html-sync <site-root> watch    # sync + watch inject/ for changes
html-sync <site-root> check    # dry-run, exit 1 if stale
```

The binary operates on a site directory containing:
- `*.html` page files with marker pairs (`<!-- html-sync:NAME -->...<!-- /html-sync:NAME -->`)
- `inject/` directory with fragment source files (`<name>.html`)

## How it works

1. Scan `*.html` files in the site root
2. For each marker pair `<!-- html-sync:NAME -->...<!-- /html-sync:NAME -->`, replace the content between markers with `inject/NAME.html`
3. Write only when content changes (byte comparison)

Markers are standard HTML comments — invisible to browsers. Content between markers is real HTML that renders. No template syntax, no placeholders, no build output.

## Conventions

- Fragment source files live in `inject/<name>.html`
- Currently supports 3 hardcoded names: `head`, `body-open`, `body-close`
- Phase 1 (next): any `inject/<name>.html` becomes a syncable fragment
- Pages opt in to a fragment by including the marker pair
- Content outside markers is never touched by the tool

## Current consumers

- `workspaces/kaizen/website-v2/` — static marketing site for hi-kaizen.com
