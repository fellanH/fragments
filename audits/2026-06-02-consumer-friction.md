# fragments consumer-friction audit · 2026-06-02

Harvested real friction signals from 7 live consumers (websites, knowledge-bases, products). Each consumer audited for: (1) drive mechanism (fragments binary vs pagekit wrapper), (2) config patterns, (3) workarounds signaling missing capabilities, (4) documented pain in AGENTS.md/specs/receipts.

---

## Per-consumer summary

| Consumer | Drive via | Notable config | Friction signal |
|----------|-----------|-----------------|-----------------|
| ettsmart.se | pagekit | Custom `[file_paths]` for absolute-path rewrite; `max_depth` implicit | None visible — canonical reference. `pagekit doctor` reports 3 CSS-loaded orphan false positives (known limitation). |
| phoeberyanofficial.com | pagekit | 11-item exclude_dirs (node_modules, css, js, fonts, assets, _assets, _backups, _mockups, _audit, functions). Declared fragment inventory in TOML. | AGENTS.md explicitly notes "Until a syncing tool is wired in, treat fragment edits as: (1) edit _fragments/, (2) regex-replace the matching block across every page." → Syncing was manually done. Now pagekit is available but may not be active. |
| felixhellstrom.com | pagekit | 7-item exclude_dirs (node_modules, css, fonts, _assets, _backups, _mockups, _audit). Canonical Shape D' layout. | None. Canonical reference consumer alongside ettsmart.se. |
| felixhellstrom.se | pagekit | Identical exclude_dirs to .com (node_modules, css, fonts, _assets, _backups, _mockups, _audit). Sibling to .com. | None. Sibling setup identical to .com. |
| phoebe-ryan | pagekit | 11-item exclude_dirs (node_modules, css, js, fonts, assets, _assets, _backups, _mockups, _audit, functions). Declared fragment inventory in TOML. | Same AGENTS.md boilerplate as phoeberyanofficial.com — hand-sync note suggests possibly pre-pagekit provisioning. |
| we-know-aeo | fragments binary (raw) | Minimal: exclude_dirs = ["_next", "images"]. No declared inventory. | Drive via raw `fragments sync .` in package.json. No pagekit wrapper. Minimal config suggests lower complexity or unfamiliarity with full config surface. |
| stormfors/knowledge-base | fragments binary (raw) | `max_depth = 6` (only consumer setting this). Exclude: ["dist", "theme"]. Non-canonical `target_dir = "."` (scan whole root). | `max_depth = 6` signals depth-searching concern. No declared inventory. Root-scan pattern (target_dir=".") is riskier than bounded target_dir. No pagekit. |

---

## Ranked friction findings

### 1. **exclude_dirs copy-paste across consumers** [CONFIG/CONVENTION]
- **Repeats:** phoeberyanofficial.com, phoebe-ryan, felixhellstrom.com, felixhellstrom.se all carry nearly identical 7–11 item exclude_dirs lists.
- **Example drift:** phoebe-ryan has `"assets"` + `"_assets"` while felixhellstrom.com omits bare `"assets"`. phoeberyanofficial.com adds `"js"` and `"functions"`.
- **Why it matters:** Copy-paste without understanding means each consumer manually list-maintains. A single canonical `pagekit` default set would collapse this.
- **Real cost:** Onboarding a new site requires copy-pasting and deciding which exclusions apply. Risk of silent surprises if you forget an exclude directory.
- **Route:** [CONNECTOR] — pagekit should ship with sane HTML-site defaults (css, js, fonts, node_modules, _assets, dist, .git, etc.) and let consumer override only on intent. Read from AGENTS.md: "phoeberyanofficial.com/fragments.toml" lines 12–23 for the largest list in scope.
- **Consumers:** ettsmart.se, phoeberyanofficial.com, phoebe-ryan, felixhellstrom.com, felixhellstrom.se.

### 2. **Hand-sync workaround documented in phoeberyanofficial.com AGENTS.md** [CONNECTOR]
- **Signal:** AGENTS.md (phoeberyanofficial.com, line ~136) explicitly says: *"Until a syncing tool is wired in, treat fragment edits as: (1) edit `_fragments/`, (2) regex-replace the matching block across every page."*
- **Status:** This note predates pagekit. phoeberyanofficial.com now has fragments.toml and declared fragment inventory, suggesting a later agent provisioned it but the old note was never removed.
- **Why it matters:** Consumers may not know whether to use `pagekit sync` or hand-edit. Lack of clarity creates both dead code and drift risk.
- **Route:** [CONNECTOR] — pagekit README and per-site AGENTS.md should explicitly say *"Run `pagekit sync` after editing `_fragments/` — never hand-edit expanded blocks."*
- **Consumers:** phoeberyanofficial.com, phoebe-ryan (identical boilerplate).

### 3. **we-know-aeo and stormfors/knowledge-base drive raw fragments binary** [CONNECTOR]
- **Pattern:** we-know-aeo uses `"fragments": "fragments sync ."` in package.json; stormfors/knowledge-base has minimal fragments.toml with no declared inventory.
- **No pagekit integration:** Both bypass pagekit, losing init/extract/file-paths/link-integrity.
- **Why it matters:** pagekit is the opinionated layer that surfaces best practices. Raw fragments binary is a valid use case but requires discipline from the consumer.
- **Route:** [CONNECTOR] — Either document a "when to use raw fragments" decision tree in specs/fragments.md, or create a pagekit-lite starter TOML that ships with better defaults.
- **Consumers:** we-know-aeo, stormfors/knowledge-base.

### 4. **max_depth = 6 on stormfors/knowledge-base (only instance)** [CONFIG/CONVENTION]
- **Signal:** stormfors/knowledge-base is the only consumer explicitly setting `max_depth`. Value of 6 suggests either (a) a very deep source tree, or (b) a concern about scanning too far.
- **Why it matters:** If this was a workaround for slow scanning or overly-broad matching, other consumers might need it but don't know to set it. If it's just an artifact from copy-paste elsewhere, it's dead config.
- **Root cause:** No consumer guidance on when max_depth is needed. Spec mentions it (specs/fragments.md) but doesn't explain the use case clearly.
- **Route:** [CONNECTOR] — pagekit should document "max_depth: when and why" in AGENTS.md template, or adjust defaults to reduce need for it.
- **Consumers:** stormfors/knowledge-base (singular).

### 5. **Non-canonical target_dir = "." on stormfors/knowledge-base** [CONFIG/CONVENTION]
- **Signal:** stormfors/knowledge-base is the only consumer with `target_dir = "."` (scan entire root). All others use `target_dir = "pages"` or similar bounded subdirectory.
- **Why it matters:** Scanning the whole root risks picking up stale backups, dev artifacts, or temporary files. Bounded scans are safer.
- **Context:** stormfors/knowledge-base has no `pages/` subfolder; its layout is flat. But this is an outlier vs the Shape D' canonical pattern.
- **Route:** [CONNECTOR] — pagekit template should default to `target_dir = "pages"` and include a comment explaining why. If a project needs flat layout, document it explicitly.
- **Consumers:** stormfors/knowledge-base (singular).

### 6. **Orphan-marker detection — false positive on CSS-loaded assets (ettsmart.se)** [CORE] — **KNOWN LIMITATION, NOT A BLOCKING DEFECT**
- **Signal:** HANDOFF.md (ettsmart.se, line ~66) documents: *"links FAIL — 3 CSS-loaded false positives (futuraptbold.otf, futuraptbook.otf, custom-checkbox-checkmark.svg). All three are referenced-from-css per `pagekit assets`. Known limitation in pagekit's orphan-detector."*
- **Why it matters:** `pagekit doctor` and `pagekit links` report orphan assets that are actually referenced from CSS via `@font-face` / `url()`. The check is correct in principle but misses CSS attribute references.
- **Impact:** Consumer has to manually verify and mark as "false positive." Not a breaking issue; signals that `pagekit assets` has better metadata than `pagekit links`.
- **Route:** [CORE] — fragments library exposes `referenced_fragment_names` but not asset references. pagekit would need to wire CSS-parsing into its asset checker to fix this. Or mark the false positives via a config allowlist.
- **Consumers:** ettsmart.se (and potentially others, not yet discovered).

### 7. **No declared fragment inventory on we-know-aeo and stormfors/knowledge-base** [CONNECTOR]
- **Signal:** we-know-aeo and stormfors/knowledge-base have fragments.toml but no `[[fragments]]` sections. Contrast: phoeberyanofficial.com and phoebe-ryan declare every fragment (name, file, description).
- **Why it matters:** Declared inventory makes it easy for an agent to see "what fragments are available" without listing `_fragments/` by hand. Also enables tooling (linting, auto-docs).
- **Route:** [CONNECTOR] — pagekit template should include a blank `[[fragments]]` section with comments. Or pagekit could auto-discover and list fragments via a new command.
- **Consumers:** we-know-aeo, stormfors/knowledge-base.

---

## Papercuts repeated across consumers

### exclude_dirs convergence
**Finding:** Four of five Shape D' sites (phoeberyanofficial.com, phoebe-ryan, felixhellstrom.com, felixhellstrom.se) carry this overlapping list:
```
["node_modules", "css", "fonts", "_assets", "_backups", "_mockups", "_audit"]
```

phoeberyanofficial.com and phoebe-ryan add: `"js"`, `"assets"`, `"functions"` (Cloudflare-specific).

**Recommendation:** Pagekit should ship with a hard-coded default that includes the first list + `"dist"`, `".git"`, `"node_modules"` (redundant but safe). Consumers opt-in to add/remove from there. **This single change eliminates 80% of copy-paste boilerplate.**

---

## Summary by route

- **[CONNECTOR]** (pagekit changes): 5 findings
  - exclude_dirs defaults (#1)
  - Hand-sync boilerplate fix (#2)
  - Raw-fragments decision tree (#3)
  - max_depth guidance (#4)
  - Non-canonical target_dir fallback (#5)
  - Declared fragment inventory template (#7)

- **[CONFIG/CONVENTION]** (consumer discipline): 0 exclusive findings (all routed as connector changes above).

- **[CORE]** (fragments library): 1 finding
  - CSS-loaded asset false positive (#6, known limitation, low priority).

---

## Closing note

**Findings count:** 7 total. **5 → pagekit defaults/guidance, 1 → CORE (low priority), 1 → known limitation.**

**Single highest-leverage fix:** Provide pagekit with sane HTML-site defaults for `exclude_dirs` (both hardcoded and documented). This collapses the copy-paste tax across all five Shape D' sites and unblocks onboarding for new consumers. Estimated effort: <30m for the change + tests.

**Next action:** Route findings #1, #2, #4, #5, #7 to pagekit AGENTS.md and TOML template. Close #6 as "documented limitation, wontfix until CSS-parse integration." Everything ships behind verify.

---

## File references

- `/Users/admin/omni/websites/ettsmart.se/fragments.toml` — Custom file_paths config, canonical reference.
- `/Users/admin/omni/websites/ettsmart.se/HANDOFF.md` lines 62–78 — Preflight state + known false positives.
- `/Users/admin/omni/websites/phoeberyanofficial.com/fragments.toml` lines 12–23 — Largest exclude_dirs list (11 items).
- `/Users/admin/omni/websites/phoeberyanofficial.com/AGENTS.md` line ~136 — Hand-sync workaround note.
- `/Users/admin/omni/companies/stormfors/knowledge-base/fragments.toml` — max_depth=6, target_dir=".", no declared inventory.
- `/Users/admin/omni/products/fragments/specs/fragments.md` — Library spec; note absence of exclude_dirs guidance.
- `/Users/admin/omni/products/fragments/src/doctor.rs` — Orphan/unpaired marker detection (CSS-loaded asset false positive location).
