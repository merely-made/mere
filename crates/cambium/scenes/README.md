# Cambium scenes

The scene lane of the Cambium umbrella: a projection engine family for interactive surfaces: heterogeneous sources
(graphs, maps, timelines, instruments, meshes) projected into scenes through one
grammar.

```text
source data + relationships + signals + score
    → select and derive
    → map data to visual channels
    → solve placement
    → produce an interactive scene
```

Sources keep their native truth behind adapters; what is shared is the scene
contract, not a data model. The representation measures content; the projection
places it.

Four crates on the unpublished 0.0.4 development line. The three engine crates
share the `sceno-` stem with one function morpheme each; `scenograph` is the
published name for authoring definitions. The engine crates landed under
`crates/cambium/scenes/` on 2026-09-03 with the Cambium family, and the
authoring crate followed on 2026-09-09 when a second host forced extraction,
per the platform boundary plan's P2:
the widget lane and the scene lane share lifecycle, input, styling and host
integration, and keep their state models apart. A widget tree is retained
interaction structure; a scene is a projection of content.

| Crate | Contents |
| --- | --- |
| [sceno](sceno/) | Core contracts. `SourceRef` / `SourceIx`, `Space` / `SpaceId`, `InstanceId`, `Backdrop`, `Footprint`, `Representation`, `ProjectedItem`, `RoutedRelation`, `Region`, `Fold` (with `StandIn`, `FoldRule`, `FoldBoundary` and the derived `FoldEffect`), `Scene`, plus the persisted `Score` / `ScoreItem` / `Arrangement` / `Placement` / `SCORE_VERSION` vocabulary and the geometry types `Vec2`, `Size2`, `Rect`, `Transform2`. |
| [scenomise](scenomise/) | Choreography. `solve(&Score) -> Scene` realizes the arrangements; `relax(&mut Scene, &Relaxation)` is a dependency-free repulsion / spring / arrangement-pull pass for surfaces without their own physics sim, including static collision against collidable backdrops. |
| [scenotime](scenotime/) | Runtime. `SceneSnapshot` / `SceneTables` with tombstoned slots, `SceneEpoch` / `Revision` / `BackdropId` / `RelationId` / `RegionId` / `FoldId`, `SceneDiff` / `SceneOp` / `apply_diff` returning `ApplyOutcome`, `TransitionSpec` / `TransitionSchedule` with pure host-time sampling, `SceneTrace` (a base snapshot and labelled steps, each an optional diff plus an opaque host annotation, chain-checked and replayed with `snapshot_at`; no cursor, which is host policy), and `pick(world) -> Option<InstanceId>`. |
| [scenograph](scenograph/) | Host-neutral authoring model. Durable projection definitions, local validation, reusable definition binding and variants, deterministic JSON, public source revisions, and opaque runtime witness binding. It owns neither a widget UI, a source authority, a solver, nor a renderer. |

## Vocabulary

- `Arrangement`: `Spiral` (with `SpiralCurve`), `Grid`, `Geographic`, `Hulls`.
- `Placement`: `Ordinal`, `Cell { column, row }`, `Coordinate(Vec2)`.
- `Footprint`: `Point`, `Circle`, `Rect`, `Polygon`, `Path`.
- `Representation`: `Glyph`, `Card`, `Sprite`, `Snapshot`, `LivePane`,
  `Open { kind }`.
- `SceneOp`: add / update / tombstone for sources, spaces, backdrops, items,
  relations, regions and folds, plus `SetItemLayer`, `SetItemOrder`,
  `SetFoldStandIn`, `SetBounds`, `SetGeneration`.
- `TransitionSpec`: duration, easing, and enter / update / exit stage windows;
  `TransitionSchedule` derives stable item windows from one validated diff and
  samples them at host-supplied elapsed time.

A `ProjectedItem` carries `source`, `space`, `transform`, `footprint`,
`representation`, `layer`, `visible`, an optional `hit` shape, and an open
`channels` map of `(name, value)` emphasis pairs.

`scenograph::presentation` authors target → condition → effect rules over
projected occurrences. Its versioned `RuleSet` validates atomically and resolves
each presentation property by priority, retaining match explanations and
overridden matches. Conditions read scalar facts and occurrence-local view
state; screen-size thresholds use host-held hysteresis. Hosts resolve these
decisions before representation measurement and placement. Effects do not
write source facts, coordinates, selection or pins. The interactive study and
shared resolver fixtures live in `support/design-studies/`; this is a
declarative authoring model, with the production Rhai runner still pending the
coordinated Genet repin in the Scenograph editor plan.

A `Backdrop` carries source provenance, space, transform, footprint, an open
appearance kind, visibility, and collision participation. Backdrops paint in
table order behind graph content and remain pointer-transparent. Interactive
features stay ordinary items over the environment.

A `Fold` draws two or more member instances as one stand-in: either a member,
which stays drawn with a "+N" of the others, or a synthetic summary at the
members' centroid. The fact alone hides the members (`FoldEffect`; no item's
`visible` changes), so unfolding is tombstoning the fold. It may record how the
host chose its members (`FoldRule::Selection` or `Descendants { root, family,
direction }`) and optional boundary accounting. Two active folds never share an
instance. A scene or snapshot without folds writes no `folds` key.

`SceneSnapshot::from_dense` starts an epoch from a one-shot `Scene`;
`SceneTables::pick` resolves a world point to the topmost live instance
(highest layer, then latest explicit order, then highest stable slot), using an
item's `hit` shape when present and its footprint otherwise.

## Portable backdrop recipes

`scenomise::backdrops` compiles three version-1 `SceneInput` recipes into a
`BackdropScene`: metre grid, fretboard, and terrain with coverage. The whole
bundle carries geometry, labels, relation explanations, source references,
revision, diagnostics and unplaced readings. Export the bundle when passing
compiled readings between hosts; a bare `sceno::Scene` omits label strings.
Alternatively export the input recipe and compile it again in another host.
Metre templates disclose `feet` and `positions_per_foot` (default two, omitted
from ordinary binary recipes). Ternary feet use three positions; stress is
still supplied independently at each syllable.

The supplying application owns facts. Knot can supply syllables, stress and
rhyme tails; Woodshed can supply tuning, positions and pitch identities;
Signalman can supply coordinates, terrain, coverage polygons and their basis.
These are adapters to a shared presentation, not dependencies of the compiler.
The compiler does not perform scansion, infer radio propagation, obtain source
authority or persist application data. Missing radio positions remain unplaced;
unmeasured ground stays distinct from supplied coverage. Expected rhyme pairs
with absent or disagreeing supplied tails remain expected, with diagnostics.

The independent renderer is
`pictograph::canvas::backdrop_surface::BackdropSurface` (`canvas` feature).
Call `new`, register a host font, then `paint(width, height, &SurfaceTheme)` to
obtain a Netrender scene. `pick` uses the same fitted coordinates. Selection,
retained attention and coverage/link visibility are view state; they do not
rewrite the source. Replacement validates and compiles before changing the live
surface. Callers supply applied colors and own resource lifetimes.
`cambium::backdrop_scene_reading` provides accessible occurrence controls and
inspection, emitting selection events to its host. Repeated occurrences retain
distinct keys even when they reference the same resource.

Graphshell is the first embedding: open `tree.html?app=local`, find Graph tools,
then choose **Portable scenes**. Its existing workspace chrome contains the
three disclosed illustrative recipes, selection, coverage/link controls,
recipe-file import and JSON export. It uses the shared renderer and semantic view
with the host's applied appearance. Opening a recipe does not import its data
into the local graph. Product adapters, source writes, pan/zoom and domain
editing are outside this first embedding.

For a local debug browser check, the full unoptimized host currently needs an
explicit larger WebAssembly stack. The receipt uses the following build (this
is a debug qualification setting, not a delivery/performance profile):

```sh
cargo rustc --manifest-path ports/graphshell/web/Cargo.toml --lib \
  --target wasm32-unknown-unknown --no-default-features \
  --features product,presentation-webgpu -- \
  -C link-arg=-zstack-size=16777216
wasm-bindgen --target web --out-dir /tmp/graphshell-backdrops/pkg \
  ports/graphshell/web/target/wasm32-unknown-unknown/debug/graphshell_web.wasm
```

Serve the generated package alongside the web directory's HTML, JavaScript,
CSS and fonts on a fresh local origin. The ordinary debug stack failed during
host startup on both the local-mere and fixture routes; the explicit-stack
build's browser result is recorded separately in the qualification receipt.

Fixtures and rendering evidence live in
[`support/design-studies/backdrops`](../../../support/design-studies/backdrops).
The CPU receipt exercises the actual shared painter with Vello, including
selection pixel changes and an unmeasured-area sample. Reproduce it with:

```sh
cargo test -p scenomise --test backdrop_scenes
cargo test -p pictograph --no-default-features --features canvas --lib backdrop_surface
cargo test -p cambium --lib backdrop_scene
cargo run -p pictograph --no-default-features --features backdrop-cpu \
  --example backdrop_receipt -- /tmp/backdrop-receipt
python3 support/design-studies/backdrops/render_cpu_pngs.py /tmp/backdrop-receipt
python3 scripts/check_port_boundaries.py
```

The CPU receipt writes raw 900×540 RGBA frames, compiled bundles, input recipes
and a JSON result. CPU paint evidence, WebAssembly compilation and headed browser
presentation are separate checks; consult the receipt's qualification record for
the actual environments exercised.

The [app-adapter receipt](../../../support/design-studies/backdrops/adapters/qualification.json)
exercises the actual Knot sound-reading exporter, Woodshed Stage chord exporter
and Signalman GPX/capture exporter. Those app-owned adapters are behind their
`portable-scenes` feature. Their outputs compile and render through the shared
surface with no app-specific renderer. Signalman's current Survey evidence
contains no admitted radio/track association or propagation estimate, so its
radio identities remain unplaced and its whole plane stays unmeasured.

The shared implementation and receiving app cohorts are now integrated on
their canonical sources. Ordinary app checks use their committed lockfiles.
For reproduction of the historical pre-integration qualification,
`support/design-studies/backdrops/with_local_scene_source.py MERE_CHECKOUT`
runs the remaining Cargo arguments in the caller's app checkout against a
temporary patch of all four scene crates. It preserves one type-bearing cohort
and writes no local paths into tracked manifests. Cargo can update that app's
lockfile, as with any ordinary repin. This helper is not needed by the current
app builds. The standalone CPU example accepts an
optional input directory after its output directory, containing `metre.json`,
`fretboard.json` and `terrain.json` from those exporters.

## Sequence templates

`scenomise::template` supplies source-disclosed lanes, nested groups, integer
tick spans, expected slots, occurrence alignments and explained relationships.
The `templated_sequence` recipe uses the same `BackdropSurface` and Graphshell
producer as metre, fretboard and terrain. Metre lowers its foot groups and
syllable positions to this shared structure while retaining stress and rhyme
meaning. Woodshed's progression adapter is the second consumer: expected
Roman-numeral roles and arranged song-block spans stay separate. Open slots
and unplaced material remain explicit, with no inferred conformity judgment.
See the [decision and reproduction guide](../../../support/design-studies/backdrops/templates/README.md).

`scenomise::document::SceneDocument` binds a primary occurrence to a supplied
scene reading. Its one-MiB total and 32-linked-reading budgets include all
recipes, and all children validate before replacement. Bare recipes remain
supported. Graphshell uses two instances of the existing portable producer
to show the primary scene and the selected occurrence's related reading.
Woodshed supplies persistent Set identities, explicit role/card/rest alignment
and saved voicing facts; the host supplies no musical interpretation.

## Dependencies

`sceno` depends on `serde` alone. `scenomise` depends on `sceno`, `serde` and
`serde_json` — the last two for the solver registry it absorbed. `scenotime`
depends on `sceno`, `serde` and `serde_json`, the last for a scene trace's
opaque host annotation. `scenograph` depends on `serde` and
`serde_json` for its durable authoring wire format. No product, engine, or GPU
dependencies: the scene lane does not reach up into Cambium's widgets or down
into Genet.

The generic `scenograph` facade is still gone (platform boundary plan §1).
Every engine consumer takes the member it needs directly, and the former
facade's solver registry behind `sceno::Arrangement::Custom` lives in
`scenomise::registry`: a registry needs the score and solver contracts at once.
`scenomise::solve_via` remains the registry dispatch entry because the plain
name is its closed-form solve over named families. The `scenograph 0.0.4` name
now hosts the portable authoring model only; it is neither an engine facade nor
a runtime.

## Status

Action intents are not modelled here. `sceno` owns instance identity,
`scenotime` owns epoch and revision identity plus deterministic transition
evaluation. The consuming protocol owns the intent triple, and the host owns
the clock. Incremental signal evaluation and renderer realization are later
work.

See [the scene contract note](../../../design_docs/scenograph_docs/technical_architecture/2026-07-22_scene_contract_note.md)
and [the epoch/diff note](../../../design_docs/scenograph_docs/technical_architecture/2026-07-22_scenotime_epoch_diff_note.md).

## License

Licensed under the Mozilla Public License, Version 2.0
([LICENSE](../../../LICENSE)), as the rest of this workspace is. The
dual Apache/MIT notice this file carried until 2026-09-03 was the standalone
`scenograph` repository's and was already stale: mere has no `LICENSE-APACHE`
or `LICENSE-MIT`, and every source file in these three crates carries an
`SPDX-License-Identifier: MPL-2.0` header.
