# fragments

Core tool complete. 19 tests. Binary, config, naming all aligned.

## Decisions

- Single primitive: sync fragments. No variables, no partials, no template syntax.
- Agent-first design. `cargo install --path .`, manifest `fragments.toml`.
- Default marker prefix `fragment`, default folder `fragments/`.

## Backlog

- Nested fragments (see `specs/fragments.md`)
- Extract command (`fragments extract <name> --from <file>`)
- Reverse sync (`fragments pull`)

## Blocked

Nothing
