# Appearance: Tabard themes and typography

**Status:** 2026-10-28: cut from `2026-07-05_theme_modes_plan.md`, archived in `archive_docs/2026-10-28/`. Done: the mode type and derivation profiles (T1), the scheme pair and cheap flip (T2), the contrast axis by sheet swap (T3), per-mode custom stylesheets (T4), custom modes by a declarative calculator (T5); the appearance workshop for the library (W1 to W3); application adoption in Pelt, Graphshell, Knot, Turnstone, Woodshed, Redshank and Signalman with macOS receipts (Graphshell in Chrome); shared typography roles with Pelt; granular font roles with Cleromancy, Knot and Woodshed; the Lagrange palette exporter as an artifact. Open: the native reader adapter, the rhai calculator lane, the workshop's missing capabilities, the typography bounds and rollout, further exporters, and the baseline debt the shared gates still exclude.

**For:** Tabard as the theme and typography library, the workshop that authors themes, and the applications that consume them: Mark's theme-model decision of 2026-07-05 (a theme is seeds, a mode is a derivation profile), the 2026-10-09 adoption order (Pelt, Graphshell, Knot, Turnstone, Woodshed, Redshank, Signalman) and the granular font roles asked for on 2026-10-11 and 2026-10-12.
Not here: Windows and Linux headed acceptance of the workshop, title bar and native decorations, live screen-reader and other assistive-technology validation (including physical input, IME and OS accessibility for font roles), and the Lagrange stock-client visual load, restart and link-icon contrast (v1.21.1's loader ignores `yellow` and `magenta`), all of which are `2026-10-28_platform_qualification_plan.md`; the shared Micron presentation scope, which is `2026-10-28_smolweb_fidelity_plan.md`; projection style authoring, beside which semantic roles are researched (A11), which is `2026-10-28_projection_grammar_plan.md`; Knot's personal custody, which lies outside this plan (lines 1482-1486).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Line numbers refer to `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md`.

- Model: a theme is a set of tinct-style seeds; a mode is a derivation profile (light, dark, high-contrast light and dark, custom); a theme may attach a custom stylesheet per mode, which replaces the derived palette for that mode; custom modes are calculators over the seeds; resolution is the custom sheet for (theme, mode) else the derived palette (lines 947-961, 1029-1036).
- Contrast floors: high-contrast modes derive measurably wider text and surface contrast (7:1 floor), and syntax roles clear 4.5:1 at normal contrast and 7:1 at high contrast (lines 989-996, 1059-1060, 799-801).
- The engine-mapping section, with `set_prefers_color_scheme` and Stylo's media table, is historical and nothing here relies on it (lines 963-985).
- Three targets stay explicit: application chrome, reader appearance and an author's published stylesheet. A reader preference never rewrites source or publishes it; Micron's authored colours remain source facts; Gemtext needs no styling extension (lines 686-689).
- Ownership: Tabard keeps definitions, modes, libraries and interchange; the workshop keeps authoring state and its portable surface; Cambium owns `title_bar`; the native host owns caption adapters and the scene producer; hosts choose application appearance, reader precedence, frame policy and native commands. No consumer grows another palette engine, draft model, file format, renderer or caption-command queue (lines 600-606, 170-174).
- Authoring: previewing never activates a theme or rewrites reader content; built-ins fork; a commit refuses a destination collision or a concurrently changed source; authored definitions stay distinct from exports and from host appearance preferences; saving the applied same-ID definition does not bypass explicit Apply (lines 750-768, 156-158, 208-212).
- Font roles: Interface, Reading, Heading, Label and Code, exposed as `--tabard-font-*` properties. Unset roles preserve host declarations; named-family inventory is not per-glyph coverage or proof of an exact face; remote and frozen receipts need realized face identity and metrics; unavailable requests stay saved with a local diagnostic; explicit authored CSS keeps ordinary cascade authority; no size, minimum size, weight, line height, content or writer measure is silently rewritten (lines 1186-1209, 47-49).
- Knot's explicit writer `SourceFace` stays authoritative, and code is not a writer alias (lines 1203-1204).
- The workshop's field selectors are classes (`tabard-text-input`), not role selectors (Mark, 2026-10-10, lines 485-497).
- Lagrange's `palette.txt` controls browser UI colours, not document themes; the documented neutral ordering and status-colour meanings are kept; no claim of CSS or DTCG import or of control over a site's page theme; exporting a file and installing it are separate user actions (lines 700-711).

## Phases

### A1. Native reader adapter for Knot and Turnstone
Map existing Tabard roles into the shared reader palette and style sheet. Knot and Turnstone own selection, persistence and precedence over site-derived defaults. Knot's `appearance.rs` derives its own Tinct palette today and can reuse Tabard while keeping editor appearance separate from published CSS. The seams are `SmolwebTheme::App` and an explicit `DocumentStyleSheet`.
- Done when: one theme is selected, previewed and restored in both apps while document bytes, history, selection and viewport remain held; ordinary, visited and focused links and reader contrast overrides stay legible; unsupported roles are preserved explicitly rather than pretended.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:8-11,670-699`

### A2. The rhai calculator lane
T5 shipped the declarative lane: a custom mode is a file of id, name, declared `(dark, high_contrast)` flags and a mapping table from every chrome role to a small OKLCH transform of one seed. Graduating to the rhai lane is open only if authors need logic the table cannot express; the file shape is the compatibility floor.
- Done when: a script-computed custom mode, authored without rebuilding the app, produces a working shell theme from the active seeds (T5's own condition, met so far by the declarative lane).
- Needs: the substrate answer in Open questions.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:1038-1044,1108-1133,1147-1148`

### A3. Workshop capabilities not yet built
The workshop edits seeds, modes, stylesheets and typography, but not custom calculators; Turnstone does not yet mount it; its specimens are a read-only reader and a selectable graph. Custom calculator creation and editing, Turnstone activation, editable content, full graph workspaces, reader link activation and session accessibility, source editing and host appearance activation are separate capabilities.
- Done when: not stated; each is named open.
- Needs: A2 for calculator editing.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:623-624,743-745,884-886,944-945`

### A4. Turnstone native appearance bounds
Turnstone's adapter is published with its workshop on macOS. Its browser-specific adapter is a separate follow-up; the dirty parent-quit Save, Discard and Cancel path and each OS focus transition were not exercised natively; independent Reader and browser rollout gates lie outside the receipt.
- Done when: not stated; the three items are named unexercised.
- Needs: A1 for the reader half.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:453-455,576-580,623-624`

### A5. Typography controls and policy
Size and minimum-size controls, a global minimum-reading-size policy, a typed DTCG typography export (colour token exports keep their scope), and per-role and per-engine typography overrides with full font enumeration (the seed-palette deferral of per-role knobs in settings is the same item; do them together).
- Done when: the controls exist and measurements stay coherent with realized fonts.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:37-41,109,1190-1193`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:169-171`

### A6. Font fidelity and packaging
Exact remote and frozen face fidelity (receipts need the realized face identity and metrics, not only a family string) and bundled, openly licensed font packaging, so installed fonts can be chosen beside openly licensed defaults.
- Done when: not stated beyond the constraints on realized faces.
- Needs: A5 for coherent measurement.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:13-16,37-41,109,1190-1193`

### A7. Typography rollout beyond the first adopters
Shared authored typography is qualified with Pelt on macOS Intel and font roles with Cleromancy, Knot and Woodshed on macOS; other applications remain open. Pelt maps Interface, Heading and Label only: its Code role and loaded-document sessions have no application binding.
- Done when: not stated; Pelt's gap and the other applications are named open.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:43-53,81-85,109,1157-1161`

### A8. Knot typography residue
Knot's adoption is bounded to family mapping. Arbitrary writer faces need a resolved-face and size metric probe identity (the existing Plex shortcut is Plex-only); typed theme size and leading are retained in the theme and workshop but not consumed by Knot; GraphCanvas's 10 px annotations and fixed label estimator are outside the qualification; Knot's default 10 second native startup and unattended presentation are unqualified, and a marked-fixture-only activation grace covers synthetic runs.
- Done when: not stated; the four items are named open.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:1203-1208,1380-1388,1419-1422,1482-1486`

### A9. Woodshed typography residue
Family availability does not close Woodshed's recorded-loop missing-glyph gate. Fixed graph and fret geometry keeps the label family without remeasuring, and rehearsal filmstrip text is baseline Interface, so the Set-tray label mapping has no evidence.
- Done when: the recorded-loop glyph gate has actual fallback and paint proof.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:1208-1209,1398-1403,1460-1463`

### A10. Theme file format and packs
Swap the theme file format to TOML (it was kept format-agnostic) and rescan theme packs on demand (startup-only today).
- Done when: not stated.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:194-195`

### A11. Semantic roles, design language and high-contrast node fills
Research semantic roles and configurable composition, so theme values have a system saying how they are used when data encoding, selection and activity compete; fill, outline and badge assignments are suggestions, not defaults chosen here. The 2026-10-09 design-language record also names configurable themes with varied defaults, Emblem and Pictograph iconography, and user-selected installed fonts with openly licensed bundled defaults, without establishing implementation. Per-theme high-contrast node fills are the matching tail.
- Done when: not stated; the research runs alongside projection style authoring, and token interchange alone does not decide these meanings.
- Needs: projection style authoring (`2026-10-28_projection_grammar_plan.md`).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:13-16,23-30`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:849-851`

### A12. Further exporters and conversions
Geopard's README describes per-domain colours and a configuration directory but no theme-import schema. An exporter stays uncommitted until public docs or controlled black-box observation establish a supported file contract and a stock-client import receipt; GTK and CSS implementation details are not that contract. Font packs, arbitrary stylesheet conversion and further protocol-specific exporters need separate consumer evidence before entering implementation. This gate does not delay other adapters.
- Done when: a supported Geopard file contract and import receipt exist, and each other item has its consumer evidence.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:721-731`

### A13. Isocosm adoption
Isocosm was excluded from application adoption while its own migration proceeds.
- Done when: not stated.
- Needs: Isocosm's migration (the isocosm repository's plans).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:138-140`

### A14. Shared baseline debt
The scoped shared CPU gate has excluded one document-canvas table test (`normal_width_table_wraps_unbroken_link_inside_its_cell`, failing on unmodified origin) and ignored one real-GPU test; a prior arrangement golden is also named baseline debt.
- Done when: not stated; the debt is named and excluded from the scoped gates.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:71-76,867-872,1398-1399`

## Open questions

- What carries the mode flip now? T2 baked the light and dark pair into one sheet and flipped by `IncrementalLayout::set_prefers_color_scheme`; T3 resolved the contrast axis as a sheet swap because Stylo's servo table lacked `prefers-contrast`. genet-layout and Stylo were retired in genet `55c05d11759` (2026-08-21), so both done-conditions name a mechanism that no longer exists. From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:980-985,998-1027`
- Does the `new`-with-scheme micro-optimisation, deferred while edits to genet-layout were in flight, have any substrate left? From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:1074-1075,1130-1133,1147-1148`
- Where does the rhai lane run? The declarative calculator and mode store were landed in the retired host's theme crate and moved into Tabard by crate consolidation C2a; which scripting lane and host carry a rhai calculator now is not stated. From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:1038-1044,1130-1133`; `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:70-72`
- Who owns Tabard's scope? The S14 pass left the owning plan "opened as a question"; C2a's T1 to T5 have landed, and the old plan's own text assigns the remaining adapter and exporter work to the theme plan. From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:735-739,1149-1153`
- Which applications still lack a font-role mapping? Pelt, Cleromancy, Knot and Woodshed are qualified; the sources do not say whether Graphshell, Turnstone, Redshank and Signalman take roles. From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:45,134-135,1157-1161`
- Who runs the formal contrast review? Formal contrast, with other platforms and physical assistive technology, is stated to lie outside the macOS appearance receipt, and no gate for it is written. From: `archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:577-580`

## Findings

- 2026-10-28 (observed in the tree): Tabard is `crates/system/tabard` (`src/lib.rs` derives the Tinct palette, DTCG colour JSON, CSS custom properties and `lagrange_palette_txt()` with typed diagnostics, including `StockVersionIgnored`; `src/smolweb.rs` defines `SmolwebTheme`, re-exported by document-lanes; `src/typography.rs`, `src/workshop.rs`, `src/library.rs`). The workshop and its desktop host are `ports/tabard/workshop` and `ports/tabard/desktop`, moved there on 2026-10-26 (ownership repair plan §5); the old plan's earlier text still says `crates/cambium/tabard-workshop` (`archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:584-593,781`).
- 2026-10-28 (observed): the excluded table test lives at `crates/inker/document-canvas/src/layout/tests.rs:1123`; its pass state was not re-run for this cut.
- 2026-10-10 (carried, checked 2026-10-28): `.gitattributes:14` pins `crates/system/tabard/tests/fixtures/**` to LF, so the Lagrange golden compares byte for byte under `core.autocrlf`; the row had been left matching nothing when Tabard moved (`archive_docs/2026-10-28/2026-07-05_theme_modes_plan.md:528-537`).

## Progress

- 2026-10-28: cut from `2026-07-05_theme_modes_plan.md`; nothing new landed.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- Container and non-text roles (Quote, List) in the document style sheet, an own plan when consumer pressure appears. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:163-164`
- Derived faces: tier 1 and 2 editing (its section 7); the branding gap (three apps ship no icon, genet ships Servo's); Pictograph's manifest version against the published 0.2.0. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1321-1326`
