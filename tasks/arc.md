# fragments

Core tool complete. 27 tests. Binary, config, naming, extract all aligned. `check` validates unpaired markers; `extract` targets the canonical occurrence among same-tag siblings; `--version` flag; CI workflow.

## Decisions

- Single primitive: sync fragments. No variables, no partials, no template syntax.
- Agent-first design. `cargo install --path .`, manifest `fragments.toml`.
- Default marker prefix `fragment`, default folder `fragments/`.

## Backlog

- Nested fragments — kept deferred per `specs/fragments.md`. Trigger: a real site where sync-only granularity is insufficient and granular fragments don't solve it. Owner: Felix on next fragments-heavy build.

## Blocked

Nothing
