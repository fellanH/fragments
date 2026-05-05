# fragments

A single Rust binary that syncs shared text fragments across files. One primitive: marked regions in target files are kept identical to source files in `fragments/`. Every file is valid, self-contained content at all times.

## Install

```bash
cargo install --path .
```

## Usage

```bash
fragments <project-root> sync            # one-shot sync (default)
fragments <project-root> watch           # sync + watch fragments/ for changes
fragments <project-root> check           # dry-run, exit 1 if stale
fragments <project-root> init about.html # create new page with marker pairs
```

## How it works

Place shared markup in `fragments/<name>.html`. In any target file, add a marker pair:

```html
<!-- fragment:nav -->
<nav>This content is replaced on sync</nav>
<!-- /fragment:nav -->
```

Run `fragments sync` — the tool replaces content between markers with the corresponding fragment source file. Content outside markers is never touched. Files are only written when content actually changes.

Every file stays valid, self-contained HTML at all times. Markers are standard HTML comments — invisible to browsers. No template syntax, no placeholders, no build output.

## Example

```
my-site/
  index.html              ← contains marker pairs
  pricing.html
  about.html
  fragments/
    head.html             ← shared <head> content
    nav.html              ← shared navigation
    footer.html           ← shared footer
    cta.html              ← shared call-to-action
  fragments.toml          ← optional config
```

Edit `fragments/nav.html`, run `fragments sync`, and every page with `<!-- fragment:nav -->` markers updates. One edit, one command, full propagation.

## Configuration

Optional `fragments.toml` at the project root:

```toml
marker_prefix = "fragment"   # prefix in <!-- PREFIX:name --> markers
fragments_dir = "fragments"  # folder containing fragment source files
```

Both fields are optional. Missing file = all defaults. Different projects can use different conventions (e.g. `marker_prefix = "html-sync"` for backwards compatibility).

## Design

- **The file is the truth.** Every `.html` file is valid, self-contained HTML. No runtime resolution.
- **Output = input.** The tool writes the same format it reads. Stop using it anytime and keep your files.
- **Single binary, zero dependencies.** No `node_modules`, no package manager, no version matrix.
- **Agent-first.** Designed so AI agents can manage large static sites with minimal context: `ls *.html` for the sitemap, `ls fragments/` for shared content, one command to propagate.

## License

MIT
