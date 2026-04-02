# fragments — Arc

## Current focus

Core tool complete. All naming aligned as `fragments` / `fragment`.

## Completed

1. **Dynamic fragment discovery** — any `fragments/<name>.html` is a syncable fragment
2. **Manifest config** — optional `fragments.toml` with `marker_prefix` and `fragments_dir`
3. **Folder rename** — `inject/` → `fragments/` as default
4. **Init command** — `fragments init <file>` scaffolds a new page with marker pairs
5. **Agent instructions** — `fragments init` generates `fragments/AGENTS.md` for agent discoverability
6. **Test suite** — 19 integration tests
7. **Rename to fragments** — binary, config file, default marker prefix all aligned
8. **Consumer site migrated** — `website-v2` markers updated from `html-sync:` to `fragment:`
9. **Spec updated** — `specs/html-compiler.md` fully aligned with new naming

## Next

- Rename workspace folder from `html-sync` to `fragments`
- Consider extending beyond HTML (the tool already works on any text file with comment markers)

## Decisions

- Single primitive: sync fragments. No variables, no partials, no template syntax.
- Agent-first design: the primary user is an AI agent managing large static sites.
- Binary installed via `cargo install --path .` (on PATH at `~/.cargo/bin/fragments`).
- Manifest (`fragments.toml`) decouples conventions from binary — different projects can use different marker prefixes and folder names.
- Default marker prefix is `fragment`, default folder is `fragments/`. Old projects set `marker_prefix = "html-sync"` for backwards compat.
