# html-sync — Arc

## Current focus

Workspace just scaffolded. Source moved from `workspaces/kaizen/website-v2/tools/html-sync/`.

## Next

1. **Phase 1: Named custom fragments** — remove the hardcoded 3-name limit, scan `inject/` for all `*.html` files and sync any matching marker pair. This is the key unlock that turns the tool from "sync head/nav/footer" into "sync anything."
2. **Phase 2: Rename to `kaizen`** — rebrand binary and CLI.
3. **Phase 3: `kaizen init`, `kaizen export`** — new subcommands.

## Decisions

- Single primitive: sync fragments. No variables, no partials, no template syntax. See `specs/html-compiler.md` "Considered and deferred."
- Agent-first design: the primary user is an AI agent managing large static sites.
- Binary installed via `cargo install --path .` (on PATH at `~/.cargo/bin/html-sync`).
