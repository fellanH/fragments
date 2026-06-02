# fragments

A single Rust binary that syncs shared text fragments across files. One
primitive: marked regions in target files are kept identical to source files
in `_fragments/`. Every file stays valid in its native format at all times.

**Format-agnostic.** Markers are ordinary comments in the *target file's own
format*, so the same fragment syncs into HTML, CSS, JS, Markdown, YAML, shell,
SQL, and more — each file staying valid in its native syntax.

For HTML-specific helpers (page scaffolding, DOM-aware extraction, link
integrity), see the sibling [`pagekit`](../pagekit), which composes this core.

## Install

**Prebuilt binary** (macOS arm64/x86_64, Linux x86_64) — from the
[latest release](https://github.com/fellanH/fragments/releases/latest):

```bash
# pick the asset for your platform, then:
tar xzf fragments-*-<target>.tar.gz
install fragments ~/.local/bin/   # or anywhere on PATH
```

**From crates.io** (published as [`fragments-sync`](https://crates.io/crates/fragments-sync); the original `fragments` name was already taken — the installed command is still `fragments`):

```bash
cargo install fragments-sync
```

**From source:**

```bash
cargo install --path .
```

## Usage

```bash
fragments <project-root> sync     # one-shot sync (default)
fragments <project-root> watch    # sync + watch _fragments/ for changes
fragments <project-root> check    # dry-run, exit 1 if stale or malformed (CI gate)
fragments <project-root> list     # list fragments and how many files reference each
fragments <project-root> config   # print effective config (defaults + fragments.toml)
fragments <project-root> doctor   # health check: orphans, unpaired/duplicate markers
```

`<project-root>` defaults to `.`. Add `--json` to `check`, `list`, or `doctor`
for machine-readable output (agent/CI consumers); exit codes are unchanged.

## How it works

Place shared content in `_fragments/<name>.<ext>`. The fragment **name** is the
file stem (`nav.html` → `nav`); its extension is irrelevant to matching. In any
target file, add a marker pair using *that file's* comment syntax:

```html
<!-- fragment:nav -->
<nav>replaced on sync</nav>
<!-- /fragment:nav -->
```

```css
/* fragment:banner */
.banner { color: red; }
/* /fragment:banner */
```

```yaml
# fragment:meta
author: old
# /fragment:meta
```

Run `fragments sync` — the tool replaces content between markers with the
corresponding fragment source. Content outside markers is never touched. Files
are only written when content actually changes (byte comparison), so diffs stay
minimal.

### Supported formats

| Comment style | Extensions (built-in) |
| --- | --- |
| `<!-- … -->` | html, htm, xhtml, xml, svg, vue, svelte, md, markdown |
| `/* … */` | css, scss, less, js, mjs, cjs, jsx, ts, tsx, c, cc, cpp, h, hpp, java, go, rs, swift, kt, php, scala, dart |
| `# …` (line) | yaml, yml, toml, sh, bash, zsh, fish, py, rb, pl, r, conf, cfg, ini, env, Dockerfile, Makefile, .gitignore |
| `-- …` (line) | sql, lua, hs, elm |

Anything not in the table is invisible to fragments. Add or override formats in
`fragments.toml` (see below).

## Configuration

Optional `fragments.toml` at the project root — all fields have defaults:

```toml
marker_prefix = "fragment"    # prefix in the <prefix>:name markers
fragments_dir = "_fragments"  # folder holding fragment source files
target_dir    = "."           # root to scan for target files
exclude_dirs  = []            # subdirectories to skip
max_depth     = 5             # how deep to walk from target_dir

# Extend or override the built-in comment-syntax table.
# Key = file extension (or file name for extensionless files).
# Value = [open, close]; an empty close means a line comment.
[syntax]
njk = ["{#", "#}"]   # Nunjucks block comment
```

The underscore-prefixed `_fragments` default keeps static-site hosts (Cloudflare
Pages, Eleventy, Jekyll) from deploying the source folder.

## Design

- **The file is the truth.** Every target file is valid in its native format. No
  runtime resolution, no source/output split.
- **Output = input.** The tool writes the same format it reads. Stop using it
  anytime and keep your files.
- **Single binary.** No `node_modules`, no package manager, no version matrix.
- **Agent-first.** `ls _fragments/` for shared content, one command to
  propagate — designed so agents can manage large sites with minimal context.

## Library

`fragments` is also a Rust library. The stable high-level API is `sync_all` /
`sync_all_with`, `check_all` / `check_all_with`, `watch::run` / `watch::run_with`,
`Config`, and the `SyncHook` trait for per-target content transforms. See
`pagekit` for a reference consumer.

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE), at your option.
