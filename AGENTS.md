# fragments

Single Rust binary that syncs marker-region content across files. One primitive — marked regions kept identical to source files in `fragments/`. Format-agnostic: works on any text file with comment-pair syntax (HTML, CSS, JS, MD, YAML, etc.). Every file is valid in its native format at all times.

For HTML-specific helpers (page scaffolding, DOM-aware extraction, link integrity), see the sibling [`pagekit`](../pagekit) workspace which composes fragments core.

Tool workspace. Worker-tier per `omni-os/omni-system/packages/rules/omni/tier-architecture.md`.

## Boot

```bash
cd ~/omni/products/fragments
cargo build --release
cp target/release/fragments ~/.local/bin/fragments
fragments --help
```

Run from a site root:

```bash
cd <site-root>
fragments sync           # one-shot
fragments watch          # sync + watch _fragments/
fragments check          # CI gate, exit 1 if stale or malformed
fragments list           # fragments + reference counts
fragments config         # print effective config
fragments doctor         # orphans, unpaired/duplicate markers
```

## Charter

This binary does:

- Replaces marker-region content in target files with `_fragments/<name>` source — format-agnostic, using the target file's own comment syntax (HTML `<!-- -->`, CSS/JS `/* */`, YAML/shell `# `, SQL `-- `, …)
- Watches `_fragments/` for changes and re-syncs on save
- Reports stale, malformed, or duplicate markers with non-zero exit (CI usable)
- Lists fragments and their references; doctor surfaces orphans
- Resolves comment syntax per file extension via a built-in table, extensible through the `[syntax]` config section

This binary does NOT:

- Run a build pipeline, generate files from a schema, or render templates
- Provide format-specific helpers (HTML scaffolding, DOM-aware extraction) — see pagekit
- Touch content outside marker pairs or files outside the project root
- Resolve nested fragments (deferred — see `specs/fragments.md`)
- Apply variables, conditionals, partials, or loops (deferred by design)

## Skills in scope

- The 6 subcommands above (`sync`, `watch`, `check`, `list`, `config`, `doctor`)
- Customization via `fragments.toml` (`marker_prefix`, `fragments_dir`, `target_dir`, `exclude_dirs`, `max_depth`, `[syntax]` overrides)
- The built-in extension→comment-syntax table in `src/syntax.rs` (HTML scaffolding / DOM extraction live in pagekit, not here)

## Tools in scope

- `cargo build --release` for builds
- `cargo test` for the integration suite at `tests/integration.rs`
- `~/.local/bin/fragments` is the canonical install location

## Canon rules especially load-bearing here

- `omni-os/omni-system/packages/rules/workflow/valuable-deliverable.md` — deliverable is the working installed binary plus passing test suite; verify by running it, not by reading the build log
- `omni-os/omni-system/packages/rules/workflow/subtract-before-building.md` — every deferred capability in `specs/fragments.md` was deferred for a reason; bias toward saying no

## Rails

- Every output file stays valid in its native format at every step. No template syntax, no placeholder leakage, no source/output split.
- Files are only written when content actually changes (byte comparison). Diffs stay minimal.
- Marker pairs are standard comments in the file's format (HTML `<!-- -->`, CSS `/* */`, etc.). They never appear in rendered output.
- Backwards-compat: legacy `marker_prefix = "html-sync"` still works via config.

## .mcp.json

Empty MCP scope. This is a Rust tool workspace — no MCP servers consumed at the workspace level.

## Origin

Promoted to standalone workspace 2026-05-05 from two divergent sources:

- `workspaces/_archive/fragments/` — full chrome (README, specs, tests, workspace.yaml), v0.4.0, missing `extract.rs` and `scraper` dep.
- `workspaces/freedom-cms/crates/fragments/` (then nested under `klarhimmel/projects/`; freedom-cms was promoted to a top-level workspace 2026-05-06) — newer src/ (added `extract.rs` + `scraper` dep), bare crate shell.

Consolidated to v0.5.0 with the freedom-cms `src/` as canonical and the archive's chrome folded back in. Driven by friction insight `01KQWS3QG23S2ATAM3TMC793CP`: the installed binary lacked `Extract` because the build came from the archive while the working code lived in freedom-cms. Two trees same-version is the bug class; standalone workspace is the recovery.
