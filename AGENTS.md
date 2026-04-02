# fragments

A single Rust binary that syncs shared text fragments across files. One primitive: marked regions in target files are kept identical to source files in `fragments/`. Every file is valid, self-contained content at all times.

**Vision spec:** `specs/html-compiler.md` — the full product vision, agent-first design, and implementation path.

## Stack

- **Language:** Rust (2021 edition)
- **Dependencies:** clap 4, walkdir 2, notify 7, anyhow 1, serde 1, toml 0.8
- **Binary:** `fragments`

## Commands

```bash
cargo build --release          # build
cargo install --path .         # install to ~/.cargo/bin/
cargo test                     # run tests
```

## Usage

```bash
fragments <site-root> sync            # one-shot sync (default)
fragments <site-root> watch           # sync + watch fragments/ for changes
fragments <site-root> check           # dry-run, exit 1 if stale
fragments <site-root> init about.html # create new page with marker pairs
```

The binary operates on a project directory containing:
- Target files with marker pairs (`<!-- fragment:NAME -->...<!-- /fragment:NAME -->`)
- `fragments/` directory with fragment source files (`<name>.html`)

## How it works

1. Load config from `fragments.toml` (optional — defaults if absent)
2. Scan `fragments/*.html`, derive marker names from filenames
3. For each marker pair in target files, replace content between markers with the fragment
4. Write only when content changes (byte comparison)

Markers are standard HTML comments — invisible to browsers. Content between markers is real HTML that renders. No template syntax, no placeholders, no build output.

## Configuration

Optional `fragments.toml` at the project root:

```toml
marker_prefix = "fragment"     # prefix in <!-- PREFIX:name --> markers
fragments_dir = "fragments"    # folder containing fragment source files
```

Both fields are optional. Missing file = all defaults. This allows different projects to use different conventions (e.g. `marker_prefix = "html-sync"` for backwards compatibility with older projects).

## Conventions

- Fragment source files live in `fragments/<name>.html`
- Any `fragments/<name>.html` becomes a syncable fragment — no hardcoded list
- Target files opt in to a fragment by including the marker pair
- Content outside markers is never touched by the tool

## Current consumers

- `workspaces/kaizen/website-v2/` — static marketing site for hi-kaizen.com
