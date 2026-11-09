# Iconography Plan: More Faces and Uses for Emblem and Pictograph

**Date:** 2026-11-09
**Status:** plan (2026-11-09). Nothing built. D1 (emblem's repository hosts the
faces crate) and D2 (identity and drawing version separately) ruled 2026-11-09;
every other fork carries a provisional recommendation.

**For:** Mark, 2026-11-09: "Let's consider additional faces/uses of
emblem/pictograph?" Offered kinship faces with blazon, a text-height arm with
inline mentions, app icons with a symbol set, and SVG output, he took all four
and added: "How about a background that responds to the contents using data?
For this, pictograph would use emblem."

**Not here:** SVG output, which is emblem's own plan
(`repos/emblem/design_docs/2026-11-09_svg_plan.md`); verification faces (a face
that proves a key; see G4); face editing beyond what the directions below need
(derived faces plan §7 still holds); the canvas's move and rename (engine-split
D13 and P3).

Receipts go to `archive_docs/receipts/2026-11-09_iconography_plan_receipts.md`.

## Where things stand

- **emblem** (own repository, published 0.2.0) is an IconVG codec:
  deterministic bytes, fills that name palette slots so the decoder's caller
  re-themes without re-encoding, level-of-detail arms chosen by rendered height.
  It ships no authored icons
  ([projection scenes record](../../2026-08-23_projection_scenes_and_graph_native_platform.md),
  around line 681).
- **pictograph** has one face grammar: hash of an address to `Params` (three
  symmetries, block or diamond, primary and accent among eight palette entries,
  a 5×5 grid), encoded with a 3×3 arm below 32 px and the full arm above
  (`crates/canvas/pictograph/src/lib.rs:87-140`, `202-330`). One global
  `DERIVATION_VERSION = 3`, mixed into an FNV-1a seed (`lib.rs:160-167`). Its
  only consumer is the canvas's `Face::Derived` arm, keyed on `node.url()` and
  cached by `(DERIVATION_VERSION, address)` (`canvas/frame.rs:130-150`).
- **Engine split.** D13: pictograph shrinks to faces (about 4.4k lines) when
  the canvas leaves as `mere-cartography`; P3 moves both to Graphshell. D7:
  emblem was to fold into Graphshell "since `pictograph` is its only consumer".
  D1 below replaces that for emblem: the directions here add consumers below
  Graphshell, so emblem stays its own repository and pictograph joins it.
- The derived faces plan
  (`archive_docs/2026-10-06_completed_plans/2026-08-28_derived_faces_plan.md`)
  still governs: palette slots only (§2), a version bump is a dated decision
  (§5), editors work on parameters and never on decoded bytes, because decode
  resolves slots to colours (§7).

## Ground rules for every face

These bind each direction below.

- **G1. A face names its grammar.** A face is described by (grammar id, version,
  key domain, key). The cache key carries all four. The existing grammar becomes
  `cells5`; `derive(address)` keeps today's seed exactly, so no current face
  changes. New grammars (`heraldic`, the backdrop's `wash`) version themselves
  and never touch `DERIVATION_VERSION`.
- **G2. Identity and drawing version separately** (D2). The seed
  version decides *what* a face is (its `Params`); a drawing version per grammar
  decides *how* the arms draw it, and enters the cache key and the digest
  fixtures but not the seed. Adding an arm (direction T) then changes bytes
  without re-rolling every node's colours and shape.
- **G3. A face is a linkability handle.** A stable face derived from a key lets
  anyone who sees it correlate appearances of that key. Each consumer states
  which key its face derives from. Content addresses and release or series
  addresses are already public, so faces from them add nothing. Persona keys
  are different: an unassociated or anonymous persona (release-log D3; the
  persona-as-capability direction of 2026-10-28) gets no derived face, or one
  derived from (context, key) so two contexts do not match. A face is never
  derived from a publisher key that the publisher has not associated.
- **G4. A face is a recognition aid, not proof.** The seed is a 64-bit FNV-1a
  hash (`lib.rs:160-167`) and the visible distinctions are far fewer (eight
  primaries, seven accents, three symmetries, two forms, a sparse grid), so a
  look-alike can be ground out in moments. No surface treats matching faces as
  evidence of matching identity. A face for comparing keys between peers would
  need its own grammar with a stated bit budget and fixed contrast; it is not in
  this plan.
- **G5. Slots only, edits upstream.** Every fill names a palette entry; a
  literal colour in generated or authored geometry is a bug. Overrides store the
  parameter source (a `Params`, a `Blazon`, a symbol's builder), never bytes.
- **G6. Keys come from the canonical address.** A face anywhere (canvas, text,
  tab, row) keys on the same canonical address the canvas uses, so a node looks
  the same everywhere. Authored hrefs, aliases and relative links resolve first.

## F. Family faces (heraldic grammar, blazon as source)

**What exists.** Lineage and branching are real in three places, none drawn as
kin: release forks (the brief's "a new log whose first release names its
source", `2026-11-03_release_log_brief.md:86-87`; `fork_of` is a release hash and
an unverified hint, release-log plan around lines 303 and 741-747); isocosm
lineages, a founding unit with a band-wide lineage policy
(`isocosm/design_docs/2026-11-02_denizen_plan.md:193-195`, `301-350`; case
1052); canvas branches, case 1048, "a branch diverges by an act; pictograph draws
it". eidetic's `impresa` is named for a heraldic device but holds only
association records (`eidetic/crates/impresa/src/impresa.rs:44-60`).

**Design.** A second grammar, `heraldic`, derives from (house, member). The
house key gives the field division, an ordinary and a charge; the member key
gives only cadency marks. A shield is the house's arms plus a small difference,
so kin read as kin.

```rust
// illustrative
struct Blazon {
    field: Field,                    // Plain(T) | Per{Pale|Fess|Bend|Chevron, T, T} | Barry(T,T) | Chequy(T,T)
    ordinary: Option<(Ordinary, T)>, // Chief | Bend | Fess | Pale | Chevron | Bordure
    charge: Option<(Charge, T)>,     // a closed set of about twelve
    marks: Vec<Mark>,                // cadency, oldest first, at most two
}
struct T(u8);                        // a palette entry, never a colour
```

- **Blazon is the canonical source.** The pipeline runs one way: keys to
  `Blazon` to geometry to IconVG. Overrides and editors write the `Blazon`; this
  is the derived faces plan's §7 tier-one editing made concrete. The structured
  serde form is canonical; a text form (`per pale T2 and T5, a cross T1, a label
  T3`) is `Display` plus a parser for exactly that subset, round-tripping, a view
  and paste surface rather than a second source. Entries are named `T0` to `T7`
  so text never says "azure". Real English blazon is open vocabulary and not
  needed.
- **The rule of tincture becomes a contrast rule.** Each palette entry gets a
  class (light or dark) from a theme-side table; a charge or ordinary must take
  the opposite class to what it sits on. The deriver obeys it and `validate`
  rejects a breach (with an explicit flag for deliberate breaks). No colour is
  read. A theme that makes two entries of opposite class nearly equal still
  breaks legibility; that is the palette supplier's existing contract.
- **Marks and depth.** Six marks reduced for size (label, crescent, star, dot,
  ring, bar), at most two stacked. At a 26 px face a mark is 5–6 px; two fit,
  three are noise. Deeper generations reuse the two-mark pair and are told apart
  by their label.
- **Level of detail.** The full arm draws everything. The coarse arm keeps field
  and charge plus only the member's own mark, drawn explicitly as a bold corner
  cell: the coarse 3×3 reduction drops any cell under half filled (`lib.rs:374`),
  so a mark derived from the full grid would vanish.
- **Keys per consumer.** Release log: house is the series' log root, and a fork
  takes its source's house only as claimed kin (below). isocosm: house is the
  lineage id, cadency from birth order. Canvas branch: house is the branch
  point's source address, the mark is the divergence act's kind. A node with no
  kinship edge keeps `cells5`.
- **Claimed kin is marked.** `fork_of` is unverified, so a series could copy a
  well-known house's face by claiming a fork. Unverified kin draws with a
  distinct mark (a broken bordure) until the release log can prove the lineage.

## T. A text-height arm and inline faces

**What exists.** The encoder takes any number of arms and checks each alone
(NaN, inverted range, an arm that moves `SEL`, an empty arm:
`emblem/src/encode/writer.rs:305-343`); the decoder runs every arm whose
half-open range contains the height (`emblem/src/vm.rs:226-231`), so overlaps
double-draw and gaps draw nothing, silently. At 14 px the 3×3 arm's cells are
4.4 px, at 10 px 3.1 px: texture, not shape. No mention construct exists:
`inker::InlineSpan` has `Link` and no `Mention`, and the Knot preview renders a
link as a button with its label (`knot-editor/apps/desktop/src/document_preview.rs:73-95`);
uxtree gives each link a `Role::Link` node (`mere/crates/forme/uxtree/src/lib.rs:455-485`).
genet-livery lays out atomic inline boxes, but its replaced elements are `img`,
`canvas` and `iframe` only (`genet/components/genet-livery/src/layout.rs:2484-2495`).

**Design.**
- Three abutting arms: `[0, 14)`, `[14, 32)`, `[32, ∞)`. Fourteen keeps the
  coarse cells at 4.4 px or more and is a one-byte coordinate.
- The text arm draws one mark filling the box: a rounded square for `Block`, a
  diamond for `Diamond`, in the primary entry, with one accent figure placed by
  symmetry (a vertical bar, a dot, a diagonal). It reads only existing `Params`
  fields: 336 distinct marks, which says "that node's colour and family", not
  identity. The text name always sits beside it.
- The same arm serves tabs, breadcrumbs and list rows, selected by height alone.
- In running text the face is a square of the font's cap height, rounded to a
  device pixel, at least 8 px, its bottom on the baseline. `Host.height` is in
  device pixels, so a 2× display picks the arm for its real size.
- The face is `aria-hidden`; the mention's accessible name is the link label,
  falling back to the title, then the address, never empty.
- Contrast: a text palette is built per background surface, each entry clamped
  to at least 3:1 against it by moving lightness only; an entry that cannot
  reach it gets a one-pixel outline in the text colour.
- A mention is a `Link` whose resolved destination is a graph node (G6); no new
  span until a need appears. A link that resolves to no node gets no face.
- **Hosting without a dependency cycle.** Inker and genet supply Graphshell, so
  they must not depend on pictograph. Livery gains a generic replaced inline
  element that paints vector paths the embedder supplies (the paint-list paths
  `canvas/derived_face.rs` already produces); the host fills it with the face.
  The fallback is a raster `img` per (version, address, palette, pixel size).

## S. Authored symbols and app icons

**What exists.** Only hocket has an app icon
(`woodshed/ports/hocket/crates/hocket-genet/icons/icon.png`, via
`[package.metadata.packager]`, generator not in the tree). `Code/mac-release.sh:11`
still points at `repos/hocket`, which no longer exists. Woodshed's
`scripts/package-macos.sh:80-148` writes an `Info.plist` with no icon; Turnstone,
Knot and Signalman package nothing. The only `.icns` and `.ico` are Servo's,
left in `genet/resources/`. No Cambium window sets an icon; the one
`with_window_icon` in the tree is Pelt's hand-filled 32×32
(`ports/pelt/desktop/static_viewer.rs:274-278`). Shell activity buttons map
Unicode marks to seven kinds (`crates/cambium/cambium/src/shell.rs:149-158`) and
draw each from five CSS boxes (`shell.css:42-54`). netrender's vello-cpu renders
a `Scene` headless (`examples/backdrop_receipt.rs`).

**Design.**
- **Home.** A `symbols` module in pictograph and a packaging binary behind a
  `pack` feature (the only part that pulls vello-cpu). After D13 pictograph is
  the faces crate, and authored pictures belong beside derived ones; a new crate
  pair would spend two names for a module's worth of code.
- **Source form.** Each symbol is Rust code against emblem's `Writer`, with a
  coarse arm and a full arm. Its encoded bytes are committed as goldens and a
  test re-encodes them byte for byte; the bytes are output, never source.
- **Palette entries.** Entries 0–7 keep the face meanings. Entries 8–11 are
  ink, muted ink, surface and a reserved host seal. The palette builder grows
  from eight to twelve (`PALETTE_SPAN` and `DERIVED_FACE_COLOR_COUNT` are
  asserted equal at `canvas/palette.rs:29-30`). Mono chrome symbols fill ink
  only, replacing `currentColor`.
- **App icons** use a fixed palette per app, because operating systems show them
  unthemed. One shared frame shape carries a per-app mark; hocket's existing mark
  is redrawn so its PNG becomes derived output.
- **Packaging tool.** For an app id: decode at each required size with
  `Host { height: size }`, render through the vello bridge and vello-cpu, write
  PNG sets and `.icns` and `.ico` containers directly (both accept PNG entries),
  with a small manifest of source hash, palette and sizes. Output goes to
  `Code/output/<lane>/icons/<app>/`; builds go through the build wrapper.
  Consumers: hocket's packager list, Woodshed's script (`CFBundleIconFile` and a
  copy step), and a window icon in the Cambium winit host for every app.
- **Chrome symbols** start with the seven shell kinds and a generic `action`
  fallback, plus window controls only if client-side chrome draws them; nothing
  ahead of a consumer.

## W. Action icons over the wire

**What exists.** `AdvertisedAction` (chirograph, in the graphshell repository,
pinned by mere at `4d971f3`) has `intent, label, explanation, payload_schema,
input_form, effect` and no icon; `input_form` is the optional-field precedent
(`#[serde(default, skip_serializing_if = "Option::is_none")]`). Graphshell
renders action buttons as label text
(`crates/graphshell/graphshell-host/src/view.rs:465-481`). The only size cap in
chirograph is `MAX_RESOURCE_CHUNK_BYTES`.

**Design.**
- `icon: Option<ActionIcon>` on `AdvertisedAction`, with the `input_form`
  serde pattern so old peers are unaffected.
- `ActionIcon::Named(id)` from the shared vocabulary: zero bytes, drawn and
  themed by the host. This ships first.
- `ActionIcon::Custom(bytes)`, guest IconVG, comes later and only with all of:
  - **Reserved names.** Lock, shield, verified, permission, window controls,
    system warnings and the host's own marks are host-only; the host refuses them
    in guest actions.
  - **Mask-only drawing.** Guest colour is discarded; guest bytes draw as an
    alpha mask in one host-chosen guest tint, never the host-seal entry.
  - **A plate.** Guest icons sit in a visibly different plate that host symbols
    never use, and the ViewBox clip keeps them inside it.
  - **No trusted surfaces.** Permission, authentication and window-chrome
    regions never render `Custom`.
  - **Limits, refused rather than truncated:** 1 KiB, 64 ops, four colour
    registers, no `Call`. They need a decode-side budget; emblem has
    `Writer::op_count()` but no decode cap, so a counting sink or a VM budget hook
    comes first.

## B. A background that responds to the contents

**What exists.** No backdrop reads the view's content through emblem. Portable
scene backdrops (`sceno-backdrops`, a graphshell git dependency) render
disclosed data, "never analysis"; `backdrop_surface.rs` realizes them with a
seven-colour literal `SurfaceTheme`. `AmbientSim` (`canvas/ambient/mod.rs`) is
decorative per-frame animation that reads no graph data. The one data-responsive
ground is territory (`canvas/territory.rs`): each site claims the ground nearest
its nodes, as a per-frame grid of flat rects, capped at 2000 nodes, coloured by
category. The signals pictograph already caches include communities, bridges,
importance, recency, spectral coordinates and sites (`src/signals/registry.rs`).
`cartography::Cluster` carries members, not geometry. The canvas's face
lowering refuses gradients (`canvas/derived_face.rs:107`); the vello bridge
supports them.

**Design.** A pure function `derive_backdrop(&ViewDigest) -> Vec<u8>` in
pictograph (grammar `wash`), fed by a digest the canvas builds, painted as a new
bottom layer under territory, beside rather than instead of the existing paths.

```rust
// illustrative
struct ViewDigest {
    kinds: [u16; K],              // quantized kind histogram (media type and tag classes)
    clusters: Vec<ClusterCell>,   // at most 16, sorted by (size desc, id)
    density: u8,                  // log2 bucket of nodes per area
    activity: u8,                 // 0..=3 from the recency span
    bounds: [i32; 4],             // world rect snapped to a 64-unit lattice
}
struct ClusterCell { id_hash: u32, size: u8, cx: i16, cy: i16, hull: [(i16, i16); 8], entry: u8 }
```

- **Input** is cached facts only (held communities, sites, recency, node
  count), quantized on a lattice; per-frame physics never enters, so nodes
  drifting within a cell change nothing. Same digest, same bytes.
- **Drawing:** one radial wash per cluster (centre at the centroid, radius from
  the hull, entries from the cluster's category), a low-alpha hull contour, a
  density motif whose pitch follows the density bucket and whose form follows
  the dominant kind, and an activity tint at the wash centre. Softness comes only
  from multi-stop gradients: IconVG has no blur or noise.
- **Level of detail:** washes and hulls when zoomed out, motifs added when zoomed
  in. `Host.height` is the backdrop's rendered height (viewport times zoom), with
  host-side hysteresis around the threshold.
- **Cadence:** re-derive only when the digest changes, after the change holds
  for several frames; between changes the decoded paths are cached in world
  space and only the camera moves. A change cross-fades host-side.
- **Bounds:** at most 16 clusters, 8 hull points each, 64 paths and 4 KiB;
  past that the smallest clusters merge into one "other" wash.
- **Low salience by rule:** every entry's luminance distance from the ground is
  capped, wash alpha at territory's top tier (0.17) and motifs at half that; no
  stroke-like shape narrower than a face cell, so nothing reads as a link. No
  hit-testing.
- **What emblem buys:** a few KiB of portable, content-addressable bytes that
  re-theme by re-decode and export to SVG, instead of a per-frame paint list.
  **What it costs:** vector only, so it complements AmbientSim and the portable
  scene backdrops rather than replacing them. Territory is the real overlap; its
  re-expression as a wash waits for evidence from B3.

## Phases

Each direction runs on its own once its prerequisites land. G1 and G2 come first
because F, T and B all name grammars and versions.

- **G. Grammar identity.** Done when: the face cache key carries grammar,
  versions and key domain; `cells5` faces are byte-identical to today's, and
  the four existing digest fixtures pass unchanged; a test shows two key domains
  with the same key bytes derive different faces; the G3 and G4 statements are
  in pictograph's crate docs.
- **H. Home (D1).** Runs with engine-split P3, when D13 leaves pictograph as
  faces only. Done when: `pictograph` builds as a member of the emblem
  repository with its history; its consumers name that repository with
  `branch = "main"`; it depends on no Graphshell or Mere crate; the mere copy is
  removed.
- **F1. Blazon.** Done when: the `Blazon` types exist and no field can hold a
  colour; `Display` and `parse` round-trip over thousands of generated blazons;
  malformed text returns a typed error and never panics.
- **F2. Derivation and contrast.** Done when: `derive_kin(house, member)` is pure
  and deterministic; the house part is identical for every member of a house and
  differs across a fixture set of houses; siblings differ by mark; the deriver
  never yields a same-class pair and `validate` rejects one; the class table is a
  theme input; digest fixtures pin the grammar.
- **F3. Geometry and legibility.** Done when: `encode_blazon` emits entries only,
  with a coarse and a full arm; one byte string decodes under two palettes into
  two themes; a render at 25.92 px of one house with three children and one
  grandchild has the house recognizable and each child distinct, reviewed by
  Mark. If the second mark is not visible, the depth limit drops to one.
- **F4. First consumer.** Done when: canvas branch faces (case 1048) use
  `derive_kin`, unverified kin draws the broken bordure, and a node with no
  kinship edge renders byte-identically to before.
- **T1. Three arms.** Done when: a test walks heights 0 to 200 in half-pixel
  steps and finds exactly one arm drawing at each; every (form, symmetry) pair
  gives a distinct text-arm path set; only palette entries appear; goldens move
  by the drawing version (D2) while every `Params` stays unchanged; every face stays under the 512-byte cap.
- **T2. Contrast-safe text palette.** Done when: a palette built from a
  background meets 3:1 for every entry with hue preserved, tested against the
  light and dark grounds and their hover and selection colours.
- **T3. Inline host.** Done when: livery's vector inline element (or the raster
  fallback) places a face at cap height on the baseline, tested at 12, 14 and
  16 px fonts and at 2× device scale; livery and Inker gain no pictograph
  dependency.
- **T4. Mentions.** Done when: a link resolving to a graph node shows its face
  and label in the Knot preview and in Graphshell panes; the uxtree link node
  keeps a non-empty name and gets no node for the face; tabs, breadcrumbs and
  rows use the same sizing helper.
- **S1. Symbols and palette entries.** Done when: the palette builder supplies
  twelve entries; the seven shell kinds and `action` exist with both arms;
  goldens re-encode byte for byte; a palette swap changes pixels and not bytes.
- **S2. Packaging tool.** Done when: the `pack` binary writes PNG sets and
  `.icns` and `.ico` for an app id; the `.icns` opens in Preview and the `.ico` in
  the Windows shell on o-pc; output is byte-stable across two runs on one machine.
- **S3. Apps.** Done when: hocket and Woodshed package generated icons (the built
  `.app` shows them), `mac-release.sh` points at `woodshed/ports/hocket`, the
  Cambium winit host takes an optional window icon that Turnstone, Knot and
  Signalman set, Pelt's hand pixels retire, and the shell's CSS box icons and
  text-mark map are gone.
- **W1. Named action icons.** Done when: `ActionIcon::Named` round-trips with old
  payloads unchanged; Graphshell draws named icons beside labels; reserved names
  are refused on guest actions.
- **W2. Custom action icons.** Starts only after a decode budget exists in
  emblem. Done when: mask-only drawing, the plate and the limits hold, and
  adversarial tests (oversized, `Call` loops, literal colours, out-of-ViewBox
  paths) are refused; no trusted surface accepts `Custom`.
- **B1. Digest and bytes.** Done when: the digest is built from cached facts
  only; moving nodes within a lattice cell leaves it unchanged; insertion order
  does not change the bytes; a version bump does; the byte and path ceilings
  fail closed at their boundary; the kind source and its buckets are named.
- **B2. Paint.** Done when: the canvas paints the bytes under territory through
  a lowering that accepts gradients (the paint list gains linear and radial
  gradients, or the layer uses the vello bridge); a palette swap recolours the
  same bytes; the contrast and alpha ceilings are tested on light and dark grounds.
- **B3. Cadence.** Done when: derivations are counted across many frames of node
  and camera motion and occur only on digest changes; a cluster oscillating
  across a lattice edge and a zoom oscillating across the threshold each cause at
  most one switch; the cross-fade's alpha is monotone.
- **B4. Hosts and export.** Done when: the reference host or isocosm's
  world-graph viewer shows it behind a switch, and its bytes export through
  emblem's SVG plan with geometry matching the decoded paint list.

## Decisions

- **D1. Emblem's repository hosts the faces crate (Mark, 2026-11-09):** asked
  where emblem and pictograph live once faces have consumers below Graphshell
  (inline faces in Inker and genet text, release faces on Mere's HTTP pages, SVG
  for merelyllc.com), with family rows as git dependencies: "Emblem repo hosts
  both (Recommended)". emblem stays its own repository; `pictograph`, faces only
  after engine-split D13, moves into it as a second crate, below every consumer.
  The canvas still leaves as `mere-cartography` for Graphshell. Outcome: amends
  engine-split D7 for emblem (the family keeps one more repository than D7
  counted); the engine-split plan's C section and Graphshell list were edited
  the same day.

  Links: area The stack · amends engine-split D7 · draws on engine-split D13 · plans iconography G · summary emblem stays its own repository and takes pictograph's faces crate, because faces have consumers below Graphshell.

- **D2. Identity and drawing version separately (Mark, 2026-11-09):** asked how
  to version the text-height arm, since bumping `DERIVATION_VERSION` re-seeds
  every node's colours and shape: "Split identity/drawing (Recommended)". The
  seed version stays at 3 and decides what a face is; each grammar's drawing
  version moves bytes, the cache key and the digest fixtures only. Outcome: new;
  refines the derived faces plan §5, whose single version covered both.

  Links: area Faces · draws on derived-faces §5 · plans iconography G, iconography T1 · summary A face's seed version decides its identity; a separate drawing version per grammar moves only its bytes.

Provisional (recommended first; asked only if they become blocking):

- F: release-log house is the series root, with claimed kin marked (alternatives:
  no kin for unassociated series; a new associated-tier house derivation).
- F: overrides per (house, member) with a house-or-member flag, so a house edit
  propagates.
- T: a mention is a resolved `Link`, not a new `InlineSpan::Mention`.
- T: livery's path-painting inline element, with raster `img` as the fallback.
- S: symbols as Rust `Writer` code in pictograph, not a text or SVG import, and
  not a new crate pair.
- S: fixed per-app icon palettes, not palettes derived from each app's seeds.
- W: `Named` first, `Custom` only after the decode budget.
- B: beside territory; a separate backdrop palette of the same shape seeded from
  the same theme; world-space paths with a camera transform; extend the paint
  list with gradients rather than drawing banded rings.

## Findings

- 2026-11-09: pictograph's seed is FNV-1a 64 over the address, XORed with
  `DERIVATION_VERSION` times a constant (`lib.rs:160-167`); there is no grammar
  or domain separation, and the version re-seeds every face.
- 2026-11-09: emblem's encoder checks each LOD arm alone (`writer.rs:312-337`) and
  the decoder runs every arm whose half-open range contains the height
  (`vm.rs:226-231`); coverage and non-overlap are the generator's job.
- 2026-11-09: the coarse 3×3 arm drops any cell under half filled (`lib.rs:374`),
  so small marks must be drawn into the coarse arm explicitly.
- 2026-11-09: the canvas face lowering refuses gradients
  (`canvas/derived_face.rs:107`); the vello bridge lowers linear and radial
  gradients with all four spreads (`src/vello.rs`).
- 2026-11-09: `AdvertisedAction` has no icon field at graphshell `40e8c8d`
  (`crates/chirograph/src/lib.rs:135-146`); mere pins `4d971f3`, not checked out
  here, so the field order is rechecked before W1.
- 2026-11-09: `Code/mac-release.sh:11` sets `REPO="$HOME/Code/repos/hocket"`,
  which does not exist; hocket lives at `repos/woodshed/ports/hocket`.
- 2026-11-09: no `Mention` inline span exists in Inker; `K::Mention` in
  `cambium/highlight.rs:68` is a source-highlighting class only.
- 2026-11-09: genet-livery's replaced elements are `img`, `canvas` and `iframe`
  (`layout.rs:2484-2495`); a vector inline element is new engine work.
- 2026-11-09: whether `canvas/frame.rs` passes logical or device pixels as
  `Host.height` is unverified; T3 settles it.

## Progress

(none)
