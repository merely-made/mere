# Iconography Plan: More Faces and Uses for Emblem and Pictograph

**Date:** 2026-11-09
**Status (2026-11-10):** G landed (grammar identity, key domains, the full
face cache key), T1 (the text arm, drawing version 2), T2 (the
contrast-clamped text palette) and T3 (inline host: livery's `cap` unit and
vector images); T4 (mentions, guest pages included), F1 (typed blazon
source), S1 (authored chrome symbols), S2 (Hocket icon pack on macOS and
Windows), W1 (named action icons), B1 (wash digest and IconVG) and B2
(theme and paint beneath territory) landed 2026-11-10.
D1 (emblem's repository hosts the faces crate) and D2 (identity and
drawing version separately) ruled 2026-11-09, D3 (guest pages through a host
style layer) and D4 (Knot keys a face on the resolved file's address)
2026-11-10. The remaining forks carry provisional recommendations.
Emblem’s SVG output landed 2026-11-09.

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

See the [implementation receipts](../../archive_docs/receipts/2026-11-09_iconography_plan_receipts.md).

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
  `flat_icon.rs` produces); the host fills it with the face.
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
  `pack` feature that enables headless Vello CPU rendering independently of
  Canvas. After D13, the pictograph faces crate will keep authored pictures
  beside derived ones.
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
- **Packaging tool.** Decode the app icon at each required size with
  `Host { height: size }`. Lower the flat paths through the shared paint-list
  bridge and rasterize with Netrender’s Vello CPU backend. Write PNG sets,
  embed them in `.icns` and `.ico` containers, and record the source hash,
  palette and sizes in a manifest. Output goes to
  `Code/output/<lane>/icons/<app>/`; builds go through the build wrapper.
  Consumers: hocket's packager list, Woodshed's script (`CFBundleIconFile` and a
  copy step), and a window icon in the Cambium winit host for every app.
- **Chrome symbols** start with the seven shell kinds and a generic `action`
  fallback, plus window controls only if client-side chrome draws them; nothing
  ahead of a consumer.

## W. Action icons over the wire

**What exists.** Chirograph’s `AdvertisedAction` carries an optional
`ActionIcon::Named(id)` after `effect`. The field follows `input_form`’s
Serde default and omission rule. Graphshell’s HTML view and retained browser
remote controls draw known names from Pictograph’s eight-symbol catalog.

**Design.**
- Named icons send only the name. Graphshell supplies the paths and uses
  its text color. Reserved and unrecognized names leave the existing text
  button available. The label remains the button’s accessible name.
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

**What exists.** Canvas can paint an Emblem wash from held content facts.
Portable scene backdrops (`sceno-backdrops`, a graphshell git dependency) render
disclosed data, "never analysis"; `backdrop_surface.rs` realizes them with a
seven-colour literal `SurfaceTheme`. `AmbientSim` (`canvas/ambient/mod.rs`) is
decorative per-frame animation that reads no graph data. The one data-responsive
ground is territory (`canvas/territory.rs`): each site claims the ground nearest
its nodes, as a per-frame grid of flat rects, capped at 2000 nodes, coloured by
category. The signals pictograph already caches include communities, bridges,
importance, recency, spectral coordinates and sites (`src/signals/registry.rs`).
`cartography::Cluster` carries members, not geometry. Canvas lowers the
wash's axis-aligned radial brushes and rectangular fills through the
existing paint-list gradient primitive.

**Design.** Pictograph’s `backdrop::derive_backdrop` encodes a `ViewDigest`
as IconVG under the `wash` grammar. The encoder returns a typed error if
the file exceeds either limit. With the wash enabled, Canvas places it
above the ground fill and beneath territory.

- `Canvas::backdrop_digest` uses an explicit pause or the held
  analytic arrangement, in that order. It filters anchors through the
  current scope and folds. It reads fresh held communities, sites and
  visits alongside materialized MIME hints and tags. A canvas without
  anchors has no digest. Positions snap down onto 64-unit cells before any
  accumulation. The world bounds add one cell of padding on each side.
- The histogram counts document, image, audio, video, data,
  collection and other. The `collection` tag takes precedence. Otherwise
  the MIME hint supplies the class. Text, PDF, EPUB and XHTML are documents.
  CSV, TSV, JSON, XML and RDF media are data. The first five class tags
  serve as fallbacks when the MIME hint supplies no class.
- Clusters use held community, then held site, then one
  unassigned group. A held community uses its least member UUID as its
  ID. Size orders the groups, with lexical IDs breaking ties.
  Each cluster stores an integer centroid and at most eight extreme
  points. Site names hash into five category slots and each cluster uses
  its most frequent slot. Overflow merges the tail into one wash.
  Density is the integer log₂ of nodes per 16 lattice cells, clamped to
  0–7. Activity uses the span of held visits among anchored members.
  No positive span gives 0, up to one hour gives 1, up to one day gives 2,
  and longer spans give 3.
- **Drawing:** one radial wash per cluster (centre at the centroid, radius from
  the hull, entries from the cluster's category), a low-alpha hull contour, a
  density motif whose pitch follows the density bucket and whose form follows
  the dominant kind, and an activity tint at the wash centre. Softness comes only
  from multi-stop gradients: IconVG has no blur or noise.
- Washes and contours draw at every height. A
  motif grid appears from 256 device pixels. `Host.height` is the
  backdrop’s rendered height. B3 will add host hysteresis.
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
- **B2. Paint.** Landed 2026-11-10. Canvas paints the bytes beneath territory
  through the paint list's radial gradient. A palette swap recolours the same
  bytes. Tests cover the contrast and alpha ceilings on light and dark grounds.
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

- **D3. Guest pages take faces from a host style layer (Mark, 2026-11-10):**
  asked how a link in Graphshell's reading pane, guest HTML in a Genet
  document, gets its face when it resolves to a graph node: "Host style layer
  (Recommended)" over injecting an `<img>` into the guest DOM or leaving guest
  pages faceless. Graphshell adds a host-only rule per resolved link,
  `a[href=…]::before { content: url(face:…) }`; the guest DOM is untouched and
  scripts never see the face. Outcome: new; needs livery to lay out and paint
  an image in `content` (it parses text, `attr()` and counters only), and the
  reading pane to accept a host sheet.

  Links: area Faces · plans iconography T4 · summary Faces on guest pages come from a host-only stylesheet, never the guest DOM.

- **D4. Knot keys a link's face on the resolved file's address (Mark,
  2026-11-10):** asked what a Knot preview face keys on, since Knot's preview
  links resolve only to local files. Mark asked what giving Knot a graph would
  take and whether the idea of a desktop module for addressing across devices
  and apps through the mere was stale. It is not: Knot's file catalog is its
  graph ("catalog, route as overmap", 2026-10-15) and its `mere-overlay`
  feature lays a resident mere's route over it through Djinn. A catalog
  document's address is its canonical `file://` URL, which its own snapshot
  carries, so "if 1 works with that, sure": the face keys on that address.
  Outcome: new; a common address scheme for every app's graph is a cue, not
  part of T4.

  Links: area Faces · draws on Knot overmap ruling 2026-10-15 · plans iconography T4 · summary A Knot link's face keys on the canonical file address of the document it resolves to.

Provisional (recommended first; asked only if they become blocking):

- F: release-log house is the series root, with claimed kin marked (alternatives:
  no kin for unassociated series; a new associated-tier house derivation).
- F: overrides per (house, member) with a house-or-member flag, so a house edit
  propagates.
- T: a mention is a resolved `Link`, not a new `InlineSpan::Mention`.
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
  (`flat_icon.rs`); the vello bridge lowers linear and radial
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
  `Host.height` is unverified; T3 settles it. Settled 2026-11-09: the canvas
  takes no device scale; `face_layer` passes the face's side in the canvas's
  own screen pixels, which are device pixels only if the host frames the canvas
  at its surface size. Inline faces take the scale explicitly
  (`inline_face_commands`).

- 2026-11-09: G keeps the address seed exactly (FNV-1a over the address,
  XORed with the seed version), so `KeyDomain::Address` needs no tag; a context
  domain hashes the context's length, the context and the key, then XORs a tag.
  A crafted address could still collide with a context seed; G4 already says a
  face proves nothing, so no stronger separation is bought.
- 2026-11-09: pictograph still holds the canvas (`src/canvas`), so the face
  cache moved with G rather than waiting for D13.
- 2026-11-09: tinct's `gate` mixes toward white or black, which shifts OKLCH
  hue, so T2 searches OKLCH lightness at fixed hue instead, dropping chroma
  only where the gamut forces it (hue holds within 4° wherever chroma is at
  least 0.03).
- 2026-11-09: one ground always admits 3:1 (white or black reaches at least
  4.58 against any ground), so the outline fallback applies only to a palette
  shared by several grounds. On the dark and high-contrast dark themes no
  palette holds against both the page and the amber node-selection fill; a
  selected row or tab takes its own palette. On the light themes one palette
  fits all four grounds.
- 2026-11-09: livery has no device scale either: it lays out in CSS pixels and
  the renderer scales. So "rounded to a device pixel" is the host's CSS
  (`round(nearest, max(8px / s, 1cap), 1px / s)`), and the vector image picks
  its arm from its CSS height times the scale it was built for. Livery had no
  `cap` unit; it now resolves one from the font like `ch`, so the face's
  height is standard CSS rather than a face rule in the engine.
- 2026-11-10: a host writes its sheet before it knows its device scale, so
  the shared rule (`InlineFaces::rule`) rounds to whole CSS pixels. That is
  whole device pixels at any integer scale, not at 1.25 or 1.5; the arm still
  follows the real scale, which the host passes the source each frame.
- 2026-11-10: Cambium's `ScopeCrumb` is built by struct literal in Turnstone
  (`src/scope_navigation/presentation.rs:71`), so a face field would break
  Turnstone at its next Mere bump; Graphshell's scope trail has no face yet.
  The grouping breadcrumbs, built in Graphshell, do.
- 2026-11-10: Graphshell's web lock resolved `spareval` 0.2.9 to `md-5` and
  `sha1` 0.11, which its code does not build against (it uses `digest` 0.10
  with `sha2` 0.10); both are pinned to 0.10.6 there. The root lock has the
  same versions and builds `mere-rdf` only when something asks.
- 2026-11-10: livery's `content` held text, `attr()` and counters only; an
  image in generated content (CSS Content 3 `url()`) was new box generation,
  layout and paint, which D3 needed (landed, genet 3fa3ec62506).
- 2026-11-10: a margin on a generated image breaks its owner's inline
  fragment in Livery: the link's rect becomes the image's, so its text loses
  its hit area. The face rule therefore carries no margin and sits flush
  against the link text. An owner inside an inline-block or inline-table, or
  one that is not a block or inline box (a flex or grid item's link, a table
  cell), generates no image yet; content mixing strings and an image keeps
  only the strings.
- 2026-11-10: a guest page reads its mentions once, at spawn (the engine's
  stylesheets are fixed then), so a node added while a page is open shows its
  face on the next load. The face's arm follows the device pixel ratio at
  spawn, not later zoom.

- 2026-11-10: `blazon::T` checks the same eight-slot range when parsing text
  and deserializing a numeric entry. `Cadency` represents zero, one or two
  marks directly, so the depth cap holds in Rust and serde as well as text.
  The contrast class table belongs to F2.

- 2026-11-10: IconVG flat fills address slots 0–15 relative to the
  selector. At the default selector of 56, slot 15 reaches face entry 7.
  The symbol encoder advances the selector by eight before either LOD
  branch, so slots 8–11 can reach the four chrome entries, then restores
  it after the branches. Neither arm changes the selector.

- 2026-11-10: Canvas and the packer can share the flat IconVG sink because
  the sink takes a resolved palette and a target rectangle. It now lives in
  `flat_icon.rs` and feeds Netrender’s Vello CPU backend through the paint
  list. The pack feature can build independently of Canvas.
- 2026-11-10: macOS accepted ten ICNS representations from five point sizes
  and their 2× versions. The ICO directory carries seven standalone RGBA
  PNGs and uses zero for the 256 px directory dimensions.
  [Apple icon sizes](https://developer.apple.com/library/archive/documentation/General/Conceptual/MOSXAppProgrammingGuide/CommonAppBehaviors/CommonAppBehaviors.html).
  [Microsoft PNG icon format](https://devblogs.microsoft.com/oldnewthing/20101022-00/?p=12473/).

- 2026-11-10: the Windows Shell icon handler on o-pc decoded the exact
  retained Hocket ICO and returned the amber mark at 32 px.
- 2026-11-10: Chirograph appends the optional icon field after `effect` and
  omits it for legacy actions. JSON compatibility preserves the old wire
  bytes. Rust producers using an `AdvertisedAction` struct literal supply
  the new field when they repin.

- 2026-11-10: existing canvas membership retirement prunes both held
  position sets. The channel registry can also expose held absolute visit
  times without changing its public normalized recency summary. B1 uses
  the absolute times only for the anchored members in scope.
- 2026-11-10: the wash encoder uses scratch registers for gradient stops
  and stores both grammar versions in two unused registers. Color sources
  refer to the host palette. Both versions affect file identity even when
  the drawing is identical.

## Progress

- 2026-11-09: G landed in pictograph: `src/grammar.rs` (`Grammar`, `CELLS5`
  with seed version 3 and drawing version 1, `KeyDomain::{Address, Context}`,
  `FaceKey`, `derive_in`, `params_in`); `derive` and `params_of` route through
  the address domain; the canvas's derived-face cache keys on `FaceKey`; the
  crate docs state G3 and G4. The four digest fixtures pass unchanged; new
  tests show every route to an address face gives the same bytes, two domains
  give two faces, and a drawing bump misses the cache without moving
  parameters.
- 2026-11-09: T1 landed in pictograph: `src/text_arm.rs` draws one plate
  (rounded square for `Block` with quadratic corners, diamond for `Diamond`) in
  the primary entry and one accent figure by symmetry (bar, dot, diagonal),
  every coordinate an integer in `[-30, 30]`; arms are `[0, 14)`, `[14, 32)`,
  `[32, ∞)` (`TEXT_THRESHOLD = 14`); `CELLS5` drawing version 2. Tests: a walk
  of heights 0 to 200 in half-pixel steps matches exactly one arm's reference
  calls at each; the six (form, symmetry) marks are distinct; the text arm is
  primary then accent; two faces agreeing on form, symmetry and colours draw
  the same mark whatever their grids; fills at 8, 20 and 64 px are palette
  entries only. The four goldens moved in length and digest only; their
  filled-cell counts did not. The largest fixture face is 217 bytes.
- 2026-11-09: T2 landed in pictograph: `canvas/text_palette.rs`
  (`TextFacePalette::new(faces, grounds)`, `TEXT_FACE_CONTRAST = 3.0`) moves
  each entry to the nearest OKLCH lightness holding 3:1 against every ground,
  keeping hue and alpha, and marks an entry `outlined` when no lightness does.
  Tests over the light, dark and both high-contrast themes, each against bg,
  surface, hover and the selection fill: every entry holds on every single
  ground; a moved entry keeps its hue and one step nearer would fail; a row's
  resting and hover grounds share one palette; outlined is exactly "below the
  floor"; alpha is kept and no grounds changes nothing.
- 2026-11-09: T3 landed. genet c1d25be4a79: livery's CSS `cap` unit (the
  font's cap height, or its ascent where it gives none) and embedder vector
  images (`VectorImage`, `LiveryDocument::set_vector_image`): an `<img>` naming
  one is laid out at its natural ratio and painted by its own paths, ahead of
  raster bytes under the same URL. Its test places a vector square inline in
  Ahem text at 12, 14 and 16 px and at 1× and 2× rounding: the box is the cap
  height rounded to a device pixel, square, bottom on the baseline, painted
  once into its own box; `cap` reads the font (16 px at 20 px Ahem, not the
  0.7 em fallback); removing the image leaves no stale paths. Livery's and
  genet-livery's 1032 tests pass. Mere: `canvas/inline_face.rs`
  (`inline_face_style`, the one sizing helper; `inline_face_commands`, the
  face's paths with the arm chosen by device height). Livery and Inker take no
  pictograph dependency; the face reaches livery only through a host's
  `VectorImage` (T4).
- 2026-11-10: T4 landed except guest pages (D3). Mere: `cambium-rootstock`'s
  `HostOptions::vector_images`, a source asked for each `<img>` URL with the
  layout scale (zoom included), reaches layout and paint through
  `HostImageLookup`; `OwnedLayout::with_vectors`; the scale follows zoom and
  display moves each frame. Cambium: `face` (`img.face`, empty `alt`,
  `aria-hidden`), `TabItem::with_face`, `frisket_with_faces`,
  `shell_face_control`. Pictograph: `InlineFace` (a `VectorImage`, with an
  outline in the text colour where the palette marks an entry outlined),
  `InlineFaces` (resolves `face:<hex address>` URLs per scale, made once),
  `inline_face_url`, `inline_face_address`, `InlineFaces::rule`, the one
  sizing rule every host's sheet carries. Graphshell: workbench tabs, the
  document rail's rows and the grouping breadcrumbs show their node's face;
  faces use the canvas's face palette held against the page ground and are
  rebuilt when the appearance changes. Knot: preview links that resolve to a
  readable local file show that document's face (D4), keyed by
  `knot_document::file_address`, now public. uxtree: a link's name is its
  label, else its title, else its address, and a face adds no node. Tests:
  host options to layout and paint at scale 1 and 2, the tab face, frisket
  faces per tile, URL round trip and refusal, one face per address and scale,
  the outline only when asked, link names, the occurrence address; Knot's
  preview test covers a resolving and a missing link.
- 2026-11-10: T4's guest pages landed (D3). Genet 3fa3ec62506 and
  7c6ffc87b9a: a `::before` or `::after` whose `content` is a single `url()`
  image is a replaced atomic inline box, sized as an `<img>`, painted from the
  embedder's vector image or bytes, adding nothing to the accessible name
  (five livery tests at 12, 14 and 16 px, unregistered, removed, mixed,
  name); `LiverySessionEngine::with_host_layer` asks the host once per spawn
  for sheets and vector images from the parsed DOM and document URL. Mere:
  Pelt's `sessions_with_host_layer`; Graphshell's
  `retained_content::mentions` turns each `<a href>` whose resolved address is
  a graph node into one host rule and one face, and the reading pane refreshes
  the graph's addresses, faces and device pixel ratio before each spawn.
  Tests: only node links get a face, one rule per href, a crafted href cannot
  leave its selector, and through the real content host a link naming a node
  is wider than the same words naming nothing while the guest bytes stay
  untouched.
- 2026-11-10: F1 landed in pictograph's `blazon` module. The structured
  source supports plain and patterned fields, six ordinaries, twelve
  charges and six cadency marks. The text form uses exact slot names and
  orders its figures as ordinary, charge, then oldest-to-newest cadency.
  Parse failures distinguish field syntax, slots, figures, figure order
  and excess marks.
- 2026-11-10: S1 landed in pictograph's `symbols` module. The palette builder
  retains the eight face entries and adds ink, muted ink, surface and the
  reserved host seal. Read, map, inspect, trail, search, tabs, fit and action
  each use ink alone, with a coarse arm below 32 px and a full arm from 32 px
  up. Each has a byte-for-byte fixture. A software raster receipt shows the
  eight symbols and three `cells5` faces in two palettes at three sizes each.
  Host UI rendering and app packaging still have their own gates.
- 2026-11-10: S2's `pack` tool writes a complete package for the `hocket`
  app ID. Its artwork redraws the amber ring and dot on the shared frame
  with a bolder mark below 32 px. The tool publishes nine PNG sizes, both
  platform containers and a source manifest together. Preview opens the
  ICNS and same-machine output is byte-stable. Windows Shell acceptance
  on o-pc remains open after connection timeouts.
- 2026-11-10: S2’s Windows Shell check passed after o-pc became reachable
  again. Shell returned the Hocket mark from the same ICO that
  was qualified through ImageIO on macOS.
- 2026-11-10: W1 landed. Chirograph `7b913edc` owns the wire field and both
  Mere lockfiles pin it. Graphshell draws host-authored symbols beside
  labels in the HTML projection view and the retained browser remote
  controls. A single catalog lookup governs both surfaces. The live
  projection fixture advertises the generic action symbol.
- 2026-11-10: B1 landed. The Canvas adapter reads held facts and emits an
  integer digest. The wash encoder emits radial gradients, hull contours
  and a detail motif from the dominant kind and density. It checks 64
  subpaths and 4096 bytes across the whole file. B2 owns palette
  qualification and painting under territory.
- 2026-11-10: B2 landed. `Canvas::set_content_backdrop` enables the wash
  in the normal frame path. The snapped bounds and ground camera project
  the wash and supply its raster height. The palette uses the canvas
  categories and edge ink, with relative luminance distance at most 0.183
  for each entry, wash alpha at most 0.17, and motif alpha at most 0.085.
  The pinned paint-list translator copies gradient stops directly into the
  premultiplied scene representation, so the wash passes Emblem's
  premultiplied channels at that seam. B3 owns derivation cadence and
  transitions. B4 owns host switches and export.
