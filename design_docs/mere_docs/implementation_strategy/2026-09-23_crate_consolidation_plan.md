# Crate consolidation plan

**Date:** 2026-09-23

**Status:** in progress, 2026-09-23. C1-C3's clear-cut moves are landed and
pushed. Mark ruled every open question the same day (see Rulings): C4's folds
are in progress in this session, insigne's delegation split is the dramatis
session's, and chatelaine waits on a CXF-shaped taxonomy.

## The ruling this serves

Mark, 2026-09-23, reviewing an audit of the workspace's publishable crates:

- The reserved crates (`chatelaine`, `dramatis`, `insigne`, `mere-alembic`,
  `mere-apparatus`, `mere-athanor`, `mien`) exist to receive their
  capabilities. Capability that grew in a crate not meant for it moves into
  its named home; stubs are not left standing beside the kludge.
- A component is a module, not a crate. Components are folded into their
  parent crate rather than kept, published or given repositories of their
  own. No new crate or repository is proposed for something that is a
  component.
- The `sibylla` and `vates` shims, and their names, are killed.

The same day's version-baseline pass (a one-time bump, then lockstep workspace
versions at the first release) waits on this plan, so the baseline is set on
the consolidated set of crates rather than churned twice.

## Phases

### C1. Shims out

Delete `sibylla` and `vates`, the deprecated re-export shims over
`esp::embed` and `esp::infer`, and every use of their names in live code.

*Done when:* no workspace member, manifest, environment variable or test
instruction names either crate; historical docs cite them as history.

### C2. Named homes filled

For each reserved crate, find where its capability lives and move it in, in
the direction that makes no dependency cycle.

| Home | Capability | Source | State |
|---|---|---|---|
| `mere-athanor` | the furnace passes: forgetting, image cleanup, consolidation, retirement | `pandect::athanor` | landed `1bda73d5` |
| `mere-alembic` | the three memory levels, behind its `recall` feature | `pandect::memory_levels` | landed `1bda73d5` |
| `mien` | standing (event grammar, ledger, persona chains and vault, gate, wire, store) and the composite reputation lens | `gemot::moot::standing`, `moothold::concord` | landed `a1551086` |
| `chatelaine` | the secret-item taxonomy, shaped against CXF's credential kinds | castellan's secret-free OTP item types, once the taxonomy exists | ruled: design first |
| `insigne` | presentable proofs: typed keys, delegation certificates and revocations, derived-key attestations | gaz's `TypedKey`; personae's delegation data types | `TypedKey` landed `0f3f854f`; delegation split ruled, in progress (dramatis session) |
| `tabard` | theme and stylesheet authoring over tinct, illume and CSS | registry's theme module, the smolweb palettes, Pelt's theme persistence | ruled 2026-09-24; see C2a |
| `dramatis` | the tier facade | none misplaced | nothing to move |
| `mere-apparatus` | the inspector pane | already its own code | nothing to move |

*Done when:* every row is landed, ruled out by Mark, or found to have
nothing misplaced, and each landed move carries its tests with it.

### C2a. Filling tabard

tabard's own first slice said what it lacked: "no host theme struct, icon
policy, syntax palette, persistence, or Pelt preview". The assessment of
2026-09-24 found each of those grown somewhere else, and Mark ruled the same
day:

- **Registry's theme module moves in, all of it:** `ThemeTokenSet`, the seed
  derivation, the custom-mode calculators and their mode files, `ChromeTheme`,
  the edge-style tokens and `ThemeRegistry` (about 2,400 lines), plus lens's
  `ThemeData`. registry loses its `theme` feature, and its lens takes
  `ThemeData` from tabard. It is tabard's own charter ("Tinct seeds in, typed
  design tokens out") run in parallel beside tabard's `Theme`.
- **`Color32` merges into tinct's `Srgb`.** The theme code's only tie to the
  graph kernel was this 4-byte RGBA struct, used in 9 files. `Srgb` holds the
  same four straight-alpha bytes under the name CSS Color 4 and DTCG give the
  space, and tabard already emits DTCG's `srgb` colour object, so Mark ruled
  a merge rather than a second type in tinct.
- **The smolweb palettes get one definition, in tabard.** `SmolwebTheme` and
  `SmolwebPalette` were defined identically in cambium-nematic and in
  document-lanes (pelt re-exports document-lanes').
- **tinct keeps deriving the syntax palette**; tabard's output carries it, and
  tabard owns the theme-file and mode-file formats.
- **Persistence moves in too**, since not everything that wants theming runs in
  Pelt.
- `document-canvas`'s `DocumentStyleSheet` stays where it is, a consumer of
  tabard's tokens.

**Persistence scope (2026-09-24).** Two stores keep a theme choice today:
Pelt's `appearance.rs` (a `Dark`/`Light` enum in a one-line file, written
atomically with `ReplaceFileW` on Windows) and pandect's `ApplicationSettings`
(`theme_id` and `theme_mode`, persona-synced opt-in, read by nothing yet).

- *Moves to tabard:* the choice itself, as a theme id plus mode rather than a
  two-value enum (the built-in ids cover dark and light); the store seam
  (current choice, set it, whether it persists); the in-memory store; and the
  atomic file store.
- *Stays in Pelt:* how Pelt presents the choice (labels, the
  `pelt-theme-*` classes, action ids) and the settings-panel adapter, which
  projects tabard's store through `mere-surface-api` and `workbench`, host
  contracts tabard should not carry.
- *pandect:* `ApplicationSettings` adopts tabard's choice type with the same
  serialized field names, so stored records read unchanged.
- *Compatibility:* Pelt's existing one-line appearance file keeps reading.

Phases, each gated like C3:

- **T1. `Color32` merged into `Srgb`.** *Done when:* the nine files use
  tinct's `Srgb`, graph-kernel's `color` module is gone, and their suites
  pass.
- **T2. The theme module and `ThemeData` into tabard, as one model.** Mark
  ruled the layout and the unification on 2026-09-24. Three steps, each gated
  on its own:
  - **T2a. Move.** Registry's tree becomes `tabard::theme` with the same
    submodules; the inner `theme` file, which holds `ThemeRegistry`, is
    renamed `registry`, and lens's file becomes `tabard::theme::data`. The two
    identical sets of theme ids become one. *Done when:* tabard owns the tree,
    registry has no `theme` feature, gloss and lens import from tabard, and
    every moved test passes.
  - **T2b. One authored theme.** tabard's `Theme` (`name`, `seeds`) is a
    strict subset of registry's `ThemeDef` (`id`, `name`, `source`, `seeds`,
    `high_contrast`, `harmony`, per-mode stylesheets). They become one type,
    `Theme` at tabard's root, with `ThemeDef`'s fields and the DTCG, CSS and
    Lagrange exports; `ThemeDef` retires and `ThemeTokenSet` stays the derived
    set. *Done when:* saved theme files read unchanged and Pelt's preview
    builds with an id.
  - **T2c. Lens themes from the token sets.** Lens's four hard-coded
    `ThemeData` values and their resolver retire; a lens's theme comes from
    the token set's derived `theme_data`, so lens colours become the
    seed-derived ones. *Done when:* saved lens files still read, by theme id
    and by value, and the lens tests assert against the token sets.
- **T3. One smolweb palette, in tabard.** *Done when:* both crates and Pelt
  use tabard's definition and their suites pass.
- **T4. Persistence into tabard.** *Done when:* Pelt's choice and stores live
  in tabard, Pelt's adapter wraps tabard's store, pandect's record uses
  tabard's type, and an existing appearance file and settings record both
  read back.
- **T5. The syntax palette and file formats in tabard's output.** *Done
  when:* tabard's tokens and stylesheet carry tinct's syntax roles.

Finding: pandect's `atomic_file::write_bytes_with_backup` and Pelt's
replace-file writer are two atomic-write implementations; T4 moves Pelt's into
tabard, and unifying the two is recorded rather than done here.

### C3. Components folded

Fold crates whose only role is a component of one parent, where no
repository outside Mere imports them.

| Fold | Into | State |
|---|---|---|
| nine `register-*` crates | `mere-registry` (`registry`), one feature per registry, `publish = false` | landed `3430ba2b` |
| `mere-trail`, `mere-glossary`, `mere-subgraph`, `mere-roster`, `mere-gloss` | modules of `mere` under the features that re-exported them | landed `61894570` |
| `mere-capability` | `servitor::cap`, replacing servitor's re-export module | landed `5335a869` |
| `eidetic-https-fetcher`, `eidetic-iroh-fetcher` | eidetic features `https-fetcher`, `iroh-fetcher` | landed `254b23b6` |

*Done when:* each fold compiles feature by feature, every moved test passes
in its new home, the portable verify gate passes, and docs citing the old
paths are repaired or annotated as historical.

### C4. Candidates with outside consumers or chosen names

Ruled by Mark 2026-09-23:

| Candidate | Ruling | State |
|---|---|---|
| `titulus` | fold into chirograph | landed `83feb122` |
| `eidetic-fjall` | fold into eidetic as a `fjall` feature (Turnstone changes its imports) | landed `3943874f` |
| `mere-resident`, `mere-mesh-host` | fold both into distillery; djinn already depends on distillery, so djinn would have made a cycle | landed: `distillery::lifecycle`, `distillery::mesh_host` |
| `graphshell-stdio`, `graphshell-local`, `graphshell-network` | fold into `graphshell-endpoint`, one feature per carrier (five repositories change imports) | landed `0c65a9d6`: `graphshell_endpoint::{stdio, local, network}` |
| `mere-canvas`, `mere-signals` | fold into `pictograph`, not the reverse; canvas becomes `pictograph::canvas` and the mere facade keeps exposing it at `mere::canvas`, so consumer paths do not change | landed: features `canvas` and `signals` on pictograph |
| `scenograph` | stays a crate: its scene editing is useful beyond graphshell | ruled out |
| `tabard` | stays a crate: the named home for theme and stylesheet authoring (C2) | ruled out |

*Done when:* each ruled fold has landed with its tests and gates.

### C5. Version baseline, after consolidation

Mark's rulings: a one-time baseline of genet 0.6 and Mere 0.4 once the
dramatis tier lands (insigne's delegation split and chatelaine's CXF-shaped
taxonomy), so those crates publish once with their real contents; genet's forks (`genet-taffy` 0.14.0,
`genet-parley` and `genet-fontique` 0.10.0) keep their upstream-derived
numbers and are listed with their own republish rule; lockstep
`version.workspace` follows at the first release.

### C6. Registry names

Nine names remain on crates.io with no crate behind them: `sibylla`, `vates`,
`mere-capability`, `titulus`, `mere-resident`, `graphshell-stdio`,
`graphshell-network`, `mere-canvas` and `mere-signals`. The other folded
crates (the `register-*` set, the five `mere` modules, the eidetic fetchers,
`eidetic-fjall`, `mere-mesh-host` and `graphshell-local`) are not on
crates.io. Deletion is done on the site by Mark and feeds the workspace-wide
crate inventory at the Code root.

## Findings

- 2026-09-23. `mere-athanor` was a 35-line stub while Athanor's 848-line
  forgetting pass sat in `crates/system/pandect/src/athanor.rs` *(historical citation)* <!-- doc-audit: historical-path -->. Its only
  outside user is Turnstone's `src/recycle.rs`. Dependencies now run
  athanor -> pandect -> alembic.
- 2026-09-23. `gemot::moot::standing` imported nothing else from gemot, so it
  moved whole. `RepLens` became generic over the moot key: mien sits below
  gemot and cannot name gemot's `MootId`. gemot's deprecated `tessera`
  module (a source alias for standing) had no user in any repository and is
  deleted; the on-disk `tessera.redb` read path stays, since it serves data.
- 2026-09-23. servitor's `cap` module was a compatibility re-export of
  `mere-capability`, a leaf split out so servitor and gemot shared one
  definition. gemot already depended on servitor, so the leaf separated
  nothing.
- 2026-09-23. The dramatis session (M0.5, gaz) established the C2 limits:
  castellan's sealed OTP store must stay in castellan, because its
  crate-private release path is what makes the gate the only way to mint a
  code, so only secret-free item types could move to chatelaine. insigne's
  code is personae's (`delegation.rs`, `provider.rs`'s
  `DerivedKeyAttestation`), so moving it makes personae depend on insigne,
  and gaz's no-cryptography rule needs insigne's core serde-only with
  verification behind a feature.
- 2026-09-23. Mark ruled on insigne: gaz's `TypedKey` moves into insigne, and
  insigne's core is plain serializable data with verification behind a
  feature. Moving personae's delegation certificates too would reverse a
  recorded ruling: the
  [device grant delegation reconciliation](../technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md)
  keeps the identity proof and attenuation grammar in personae ("This needs
  no change to personae"). This plan's earlier insigne row, which named
  delegation certificates, contradicted that and is corrected. The dramatis
  session is putting the split to Mark: data types in insigne, issuing in
  personae, checking behind insigne's feature.
- 2026-09-23. The first fold tool rewrote sibling references after prefixing
  module paths, so a module named like its old crate produced
  `crate::crate::subgraph`; caught by the compiler in one test file and
  fixed before commit.
- 2026-09-23. The C1-C3 moves broke two relative links and 27 path
  citations across ten docs; `scripts/mere_doc_audit.py` run at
  the pre-move commit and at head isolated them.
- 2026-09-24. `Color32` serialized as a four-element array and `Srgb`
  serializes as an `r`/`g`/`b`/`a` map, so the merge changes the serialized
  shape of `ThemeTokenSet`, `ChromeTheme` and the edge tokens. Nothing stores
  or sends them: saved themes (`ThemeDef`) keep seeds, already `Srgb`, and
  custom mode files keep seed references and lightness. Nothing inside the
  kernel used its `Color32`, so the module was deleted rather than moved; the
  kernel's f32 `Color` in `paint.rs` is the renderer-side type and stays.
- 2026-09-24. Lens's `ThemeData` keeps its `(u8, u8, u8)` tuples rather
  than `Srgb`: saved lens files store them, and a map-shaped colour would stop
  them reading.
- 2026-09-24. tabard's directory carries `LICENSE-MIT` and `LICENSE-APACHE`
  from its 2026-08-10 reservation, while its manifest, source headers and
  README say MPL-2.0, and the published 0.0.1 package ships both files. Mark
  ruled them deleted the same day. tinct carries the same mismatch (a
  `LICENSE-MIT` beside an MPL-2.0 manifest), put to Mark separately.

## Progress

- 2026-09-23. C1 landed (`d69be378`). esp's model-directory variables became
  `ESP_MINILM_DIR` and `ESP_TINYLLAMA_DIR`, and esp's persistence dropped a
  private newtype that only existed to dodge the orphan rule across the old
  crate split.
- 2026-09-23. C2: athanor and alembic (`1bda73d5`), mien (`a1551086`).
  Tests moved with the code: mien 59, gemot 147 -> 97, moothold 18 -> 9.
- 2026-09-23. C3: registry (`3430ba2b`, 141 tests), mere modules
  (`61894570`, 44 tests), `servitor::cap` (`5335a869`, 8 tests), eidetic
  fetchers (`254b23b6`, 9 tests), all passing. Mere's workspace went from
  124 members to 106; the portable build from 1,549 packages to 1,531.
- 2026-09-23. Docs citing the moved paths repaired or annotated as
  historical; the doc audit reports zero broken relative links. Pushed with
  Mark's go-ahead as part of `254b23b6..03c05dbd`.
- 2026-09-23. C2 insigne ruled; the dramatis session is moving gaz's
  `TypedKey` in. The delegation-certificate question is open.
- 2026-09-24. C4's last two folds landed: the graphshell carriers
  (`0c65a9d6`) and canvas with mere-signals into pictograph (`f590e45d`).
  Mere's workspace is at 97 members, down from 124 before this plan; the
  portable build is at 1,521 packages. canvas carried an unbuilt 771-line
  winit canvas-host binary (`autobins = false`, no `[[bin]]`); Mark ruled it
  deleted. Next in this plan: tabard's fill, ruled 2026-09-24.
- 2026-09-24. T1 landed: `Color32` merged into tinct's `Srgb` in the nine
  files, and graph-kernel's `color` module deleted. Suites pass as before
  (tinct 15, kernel 290, registry 139 with 2 ignored, mere 47); registry
  checks with `knowledge`, `lens` and `theme` each alone; the portable gate
  passes. Next: T2.
- 2026-09-24. T2a landed (`7f133433`): registry's theme tree is
  `tabard::theme`, lens's theme file is `tabard::theme::data`, and one set of
  theme ids replaces two. 24 tests moved (registry 141 -> 117, tabard
  9 -> 33); tabard 33, registry 115 with 2 ignored and mere 47 pass, registry
  checks with `lens` and `knowledge` alone, and the portable gate passes. The
  `mere` facade no longer depends on registry. tabard's golden Lagrange test
  had been failing on Windows checkouts, where autocrlf rewrote its fixture;
  the fixtures are pinned to LF (`b7ed0bdc`). Next: T2b.
- 2026-09-24. T2b landed (`22ffe681`): tabard's `Theme` carries `ThemeDef`'s
  fields, serde defaults and `mode_sheet()` beside its exports, and `ThemeDef`
  is gone. `Theme::new` takes an id (Pelt's preview is `pelt:tabard_preview`),
  and `Theme` drops `Eq`, since `Harmony` carries floats. A new test reads a
  theme file in the shape saved before the merge; the older "legacy" check
  serialized a current built-in, so it never parsed a file missing a field.
  tabard 34, Pelt builds with both preview features, and the portable gate
  passes.
- 2026-09-24. T2c landed (`d70fa9cc`): lens themes resolve from the built-in
  token sets' derived `theme_data`. Default's lens accent goes from cyan
  (80,220,255) to the seeds' blue (51,102,200) and its background from
  (20,20,25) to (9,12,26); light darkens slightly; dark keeps its accent;
  high contrast is unchanged. The four derived values are distinct, so
  `theme_data_id` still round-trips, and saved lenses read by id and by
  value. tabard 35, registry 115 with 2 ignored, mere 47, and the portable
  gate passes. T2 is complete; next: T3.
