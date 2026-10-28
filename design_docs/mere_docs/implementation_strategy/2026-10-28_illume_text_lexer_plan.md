# Illume: the text lexer and its omnibar legibility goal

**Status:** 2026-10-28: cut from `2026-06-26_illume_text_lexer_plan.md`, archived in `archive_docs/2026-10-28/`. Build points 1 to 6 (tinct's syntax palette, genet's styled field, the illume rename and entity pass, one style-aware field body, the bridge, the `KnotReadout` deriver) are done. Point 7 published tinct 0.1.0 and illume 0.0.1 on 2026-06-27; point 8's extraction was reversed on 2026-09-03, and illume, tinct and cambium are workspace path members again. Open: a stable illume release, the omnibar grammar gaps, and a live consumer, since the editor and omnibar highlighting that proved the pipeline were retired with meerkat.

**For:** illume as a pure-Rust, wasm-safe lexer and highlighter paired with tinct for themed colour and a styled field for rendering, and the north-star that opened it: mere should feel comfortable and manipulable from the omnibar, with the lexer as the legibility layer of a keyboard-driven interface (Mark, 2026-06-26). This plan keeps the keyword `illume-text-lexer`, so the Decisions below are copied unchanged.
Not here: the licence rule that illume and tinct ship their new licence at each crate's next functional bump, which is the workspace hygiene plan (`2026-10-28_workspace_hygiene_plan.md`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Line numbers refer to `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md`.

- Three pieces stay separable: illume (text to `(range, SyntaxKind)` spans, crates.io-only), tinct (`SyntaxRole` to a contrast-gated colour derived from theme seeds) and the styled field (spans to styled runs). The `SyntaxKind` to `SyntaxRole` map is the only place that knows both, so illume and tinct stay mutually independent (lines 66-86; Decisions 1, 2, 5).
- A crate promotes only if it depends on crates.io plus a minimal portable patch, never on the personal mere workspace, and fills a niche another host or forker would want; host-coupled glue stays home (lines 48-64).
- The omnibar goal sharpens the control rule from "every control operable in the active input mode" to "the keyboard user can drive and read the whole thing from the omnibar, expressively" (lines 97-100).
- Highlight colours are derived from the active theme's seeds, never hardcoded, and highlight runs carry classes, not inline CSS (Decisions 1 and 2, lines 170-176).

## Phases

### L1. A stable illume release
illume 0.0.1 was published as a name-reserve straight from the workspace; the 0.0.x signals that the API is still moving. A stable release (0.1) follows once the API settles. The workspace crate is `crates/nematic/illume`.
- Done when: a stable illume release is published once the API settles; the sources state no further gate.
- Needs: the API to settle.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:12-13,145-149,312-314`

### L2. Omnibar grammar gaps
Close the three named gaps so the lexer makes the omnibar readable as you type: one shared grammar across the omnibar, so commands, paths, queries and scripts lex as one highlighted syntax; completion everywhere, not just after `[[` and `/`; and a typable graph query. The cheapest first payoff, highlighting the omnibar input live, was proven on the retired host and is not a live consumer today.
- Done when: the three gaps are closed and the keyboard user can navigate, manage the graph, run commands and scripts, and open and edit notes, all typed and all legible (the source states the goal and names the gaps, with no finer gate).
- Needs: an omnibar host that carries the lexer; none is named (see Open questions).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:13-14,88-106`

### L3. Rename `knot-editor-host` to `illume-host`
The host-side reuse-lexer crate keeps the app's own parsers (cssparser, html5ever, boa_parser, pulldown-cmark) over illume's portable floor. The rename is for clarity and mere-internal, so it was deferred.
- Done when: the crate is renamed; the sources state nothing finer.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:57-60`

## Open questions

- Which host carries highlighting now? The editor and omnibar consumers were retired with meerkat on 2026-07-18, and the archived plan names no live consumer. The omnibar goal also sat on the `>` shell and `ActionRegistry` of the archived command-registry plan; whether a successor substrate exists is not stated. From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:6-7,194-195`
- Do the stale follow-ons still stand? The 2026-07-08 notes say Isometry adopts highlighting with one `xilem-serval/highlight` flag and the omnibar re-wires as one `highlighted_text_field` call; both assume the xilem-serval extraction and meerkat, which have since gone. The real-mouse check of click-to-focus in the floated editor panel was a harness limit on a retired host. From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:286-289,338-340`
- Are the published crates still maintained? The publication of tinct 0.1.0 and illume 0.0.1 was not re-checked in the last pass (no network), and tinct is no longer sourced from crates.io. From: `archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:7-8,121-123`

## Decisions

1. **Colours are derived, not hardcoded** (#1). The host maps `SyntaxKind` → `SyntaxRole`
   → `tinct::derive_syntax_palette` over the active theme's seeds (fetched directly via
   `theme.theme_def(id).seeds`, no register-theme change needed), emitting the `syntax-*`
   rules into the chrome stylesheet genet applies. No parallel palette; tracks the theme
   and any reseed.
2. **Highlight runs carry classes, not inline CSS.** Themeable through one sheet, the
   genet/stylo way, so user/mod themes override.
3. **One style-aware field body** (#2), not a styled fork of the plain one.
4. **`KnotReadout` is a stateless deriver** (#3, renamed from `KnotEditor`), the host owns
   the buffer. The portable derivations (highlight / outline / folds / render) take text;
   the registry is built once and held, fixing the per-keystroke rebuild.
5. **The `SyntaxKind` → `SyntaxRole` seam is the host's**, keeping illume and tinct
   independent of each other. *(Revised 2026-07-08 — see Decision 8: with two consumers
   (meerkat, Isometry) the map is shared infrastructure, not host glue, so it moved into
   xilem-serval's `highlight` feature. illume and tinct stay mutually independent — the
   toolkit is the one place that knows both.)*

## Findings

- 2026-10-06 (carried): illume is `crates/nematic/illume`, licensed MPL-2.0; tinct is `crates/cambium/tinct`; the highlight bridge is cambium's `highlight` feature (`crates/cambium/cambium/src/highlight.rs`), not xilem-serval's. The Decisions section is copied unchanged, so Decision 5's 2026-07-08 revision (the seam moved into xilem-serval) still appears there, and its "Decision 8" names the old build sequence's point 8 (lines 150-158, 182); the extraction behind it was reversed on 2026-09-03 (`archive_docs/2026-10-28/2026-06-26_illume_text_lexer_plan.md:4-14,160-166`).
- 2026-10-28 (observed in the tree, not in the old plan): the workspace manifest pins illume at 0.0.3 (`Cargo.toml:352`) and tinct at 0.2.0 (`Cargo.toml:461`); `highlighted_text_field` is documented there as the single-line, entities-only mode for an omnibar (`crates/cambium/cambium/src/highlight.rs:21-23`); `ports/graphshell/Cargo.toml:39` lists illume and `knot-editor-host` under its optional `editable-blocks` feature.

## Progress

- 2026-10-28: cut from `2026-06-26_illume_text_lexer_plan.md`; nothing new landed.
