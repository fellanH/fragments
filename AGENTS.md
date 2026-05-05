# fragments

Single Rust binary that syncs shared text fragments across files. One primitive — marked regions kept identical to source files in `fragments/`. Every file is valid, self-contained content at all times.

Tool workspace. Worker-tier per `harness/rules/omni/tier-architecture.md`.

## Boot

```bash
cd ~/omni/workspaces/fragments
cargo build --release
cp target/release/fragments ~/.local/bin/fragments
fragments --help
```

Run from a site root:

```bash
cd <site-root>
fragments sync           # one-shot
fragments watch          # sync + watch fragments/
fragments check          # CI gate, exit 1 if stale
fragments init <file>    # scaffold new page with marker pairs
fragments extract        # detect duplicate blocks, extract to _fragments/, insert markers
```

## Charter

This binary does:

- Replaces marker-region content in `*.html` files with `fragments/<name>.html` source
- Watches `fragments/` for changes and re-syncs on save
- Reports stale files with non-zero exit (CI usable)
- Scaffolds new pages with marker pairs for every existing fragment
- Extracts duplicated blocks across pages into reusable fragments

This binary does NOT:

- Run a build pipeline, generate files from a schema, or render templates
- Provide a GUI, CMS, or hosting layer
- Touch content outside marker pairs or files outside the project root
- Resolve nested fragments (deferred — see `specs/fragments.md`)
- Apply variables, conditionals, partials, or loops (deferred by design)

## Skills in scope

- The five subcommands above
- Customization via `fragments.toml` (`marker_prefix`, `fragments_dir`, `target_dir`)

## Tools in scope

- `cargo build --release` for builds
- `cargo test` for the integration suite at `tests/integration.rs`
- `~/.local/bin/fragments` is the canonical install location

## Canon rules especially load-bearing here

- `harness/rules/workflow/build-not-dev.md` — release builds, not dev watchers
- `harness/rules/omni/dispatch-verification.md` — verify by running the binary, not reading the build log
- `harness/rules/workflow/valuable-deliverable.md` — deliverable is the working installed binary plus passing test suite
- `harness/rules/workflow/subtract-before-building.md` — every deferred capability in `specs/fragments.md` was deferred for a reason; bias toward saying no

## Rails

- Every output file must be valid HTML at every step. No template syntax, no placeholder leakage, no source/output split.
- Files are only written when content actually changes (byte comparison). Diffs stay minimal.
- Marker pairs are standard HTML comments. They never appear in rendered output.
- Backwards-compat: legacy `marker_prefix = "html-sync"` still works via config.

## .mcp.json

Empty MCP scope. This is a Rust tool workspace — no MCP servers consumed at the workspace level.

## Origin

Promoted to standalone workspace 2026-05-05 from two divergent sources:

- `workspaces/_archive/fragments/` — full chrome (README, specs, tests, workspace.yaml), v0.4.0, missing `extract.rs` and `scraper` dep.
- `workspaces/klarhimmel/projects/freedom-cms/crates/fragments/` — newer src/ (added `extract.rs` + `scraper` dep), bare crate shell.

Consolidated to v0.5.0 with the freedom-cms `src/` as canonical and the archive's chrome folded back in. Driven by friction insight `01KQWS3QG23S2ATAM3TMC793CP`: the installed binary lacked `Extract` because the build came from the archive while the working code lived in freedom-cms. Two trees same-version is the bug class; standalone workspace is the recovery.
