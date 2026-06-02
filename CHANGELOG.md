# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] — 2026-06-02

### Added
- **Format-agnostic comment syntax.** Markers are now ordinary comments in the *target file's own format*. A single fragment syncs into HTML/Markdown (`<!-- -->`), CSS/JS/Rust/C-family (`/* */`), YAML/shell/TOML/Python (`# …`), and SQL/Lua (`-- …`) — each target staying valid in its native format. Previously HTML-only.
- `[syntax]` config section to extend or override the built-in extension→comment-syntax table (e.g. Nunjucks `njk = ["{#", "#}"]`).
- New `syntax` module exposing `CommentSyntax` for library consumers.
- Duplicate fragment-name detection: two source files resolving to the same name (e.g. `header.html` + `header.css`) now fail with a clear error instead of silently picking one.
- `tests/format_agnostic.rs` — 9 tests covering CSS, YAML, Markdown, shell, cross-format sync, line-comment name boundaries, config overrides, and duplicate-name errors.
- Packaging metadata (`license`, `repository`, `keywords`, `categories`), dual `MIT OR Apache-2.0` license files, and a tagged-release GitHub Actions workflow producing cross-platform binaries.

### Changed
- Fragment loading is no longer restricted to `.html` files — any non-hidden file in the fragments directory is a fragment, named by its file stem.
- Target scanning now includes any file whose format has a known (built-in or configured) comment syntax, not just `.html`.
- `referenced_fragment_names` now takes a `CommentSyntax` argument (library API change; the high-level `sync_all`/`check_all`/`watch` entry points are unchanged).
- Hardened fragment-name derivation: non-UTF-8 names are skipped rather than panicking.

## [0.6.1] — 2026-05-06

### Added
- Hookable watch API via `run_with` for custom build integration.

## [0.6.0] — 2026-05-06

### Added
- SyncHook API for per-target fragment transforms.
- List, config, and doctor commands (Workstream B P2).
- User-defined extract candidates via `[[extract.candidates]]` in config.
- Exclude directories and max depth configuration in `fragments.toml`.

### Changed
- Default fragments directory is now `_fragments`.

### Fixed
- Warn on duplicate marker pairs in check and doctor commands.

## [0.5.0] — 2026-05-05

### Added
- Fragments promoted to standalone workspace.
- Split HTML helpers to pagekit, expose fragments as reusable library.
- Position fragments as format-agnostic primitive (stage 1 of pagekit fork).
- Crash-safe file writes via tempfile and rename.
- Tighten check command, fix extract targeting, add CI.

### Fixed
- Revert tempfile+rename — return to direct writes with better error context.
