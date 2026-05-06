# fragments

## Active arc

Harden fragments through real-site usage. felixhellstrom.com
(`~/omni/websites/felixhellstrom.com`) is the canonical consumer:
21 pages, 15 fragments, integrated since 2026-05-05, currently sync
clean against the latest binary.

In parallel: lift hardcoded values into `fragments.toml` so any project
can adopt fragments without source patches. Goal — drop `fragments.toml`
+ run `fragments init` + run `fragments sync`, no source edits required.

## Workstream A — felixhellstrom.com as living testbed

Status: integrated. Site at `~/omni/websites/felixhellstrom.com` runs on
fragments. Workstream is now ongoing usage, not initial wiring.

- Capture friction inline as it surfaces (vault tag: `bucket:fragments,friction`)
- Each friction entry triggers: ship a fix, kill with rationale, or queue
  with explicit trigger per `harness/rules/workflow/capture-or-kill.md`
- Watch for: extract heuristic gaps (site uses `<header>`/`<footer>` so
  default candidates work, but custom layouts will surface), unmarked
  duplicates, watch-mode latency, init template pain points

## Workstream B — third-party consumability via settings

Lift hardcoded values into `fragments.toml`. Three surfaces are the
current friction for new projects:

| Currently hardcoded | Symptom |
|---|---|
| Excluded scan dirs (`tools/ node_modules/ css/ fonts/ _assets/`) | Project with `dist/`, `build/`, or custom asset dirs gets wrong scan scope |
| Walk depth `5` | Sites organized at depth ≥ 5 silently invisible |
| Extract candidate list (6 selectors) | Site using `.brand-bar` or `.menu-primary` instead of `.navbar` gets nothing extracted |

### P0 — config replaces hardcoded values
- `exclude_dirs: Vec<String>` in `fragments.toml`, replaces hardcoded list
- `max_depth: usize` with sensible default

### P1 — extract works for arbitrary layouts
- `[[extract.candidates]]` table: list of `(name, selector)` pairs
- Built-in defaults still apply if section absent

### P2 — discoverability
- `fragments list` — show fragments + which pages reference each
- `fragments config --print` — dump effective config (defaults + overrides)
- `fragments doctor` — health report (unmarked duplicates, orphans)
- Expand `--help`, ship example `fragments.toml` in `init`

### P3 — distribution
- `cargo publish` to crates.io once config surface is stable enough to
  commit to semver

## Done-when

- felixhellstrom.com runs on fragments with no source patches needed when
  the site grows or layout shifts (Workstream A signal)
- One unfamiliar-with-source project can adopt fragments via
  `cargo install` + `fragments.toml` and run end-to-end (Workstream B)
- Friction backlog from real use is processed

## Open questions

- **Default `fragments_dir`**: spec says `fragments`, code says `fragments`,
  felixhellstrom (only real consumer) explicitly sets `_fragments`. If a
  second site adopts and also picks `_fragments`, the spec default is
  wrong. Revisit once n=2.

## Decisions

- Single primitive: sync fragments. No variables, no partials, no template syntax.
- Agent-first design.
- Default marker prefix `fragment`, default folder `fragments/`.
- Direct `fs::write` (truncation risk accepted; recovery is `sync` re-run + `check`).

## Backlog

- Beyond HTML (CSS/JS/MD via per-extension comment syntax) — deferred until
  a real cross-format need surfaces during Workstream A
- Nested fragments — deferred per spec

## Blocked

Nothing
