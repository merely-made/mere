# App-owned portable scene exports

These are actual outputs of the optional `portable-scenes` adapters in Knot,
Woodshed and Signalman, rendered by the shared `BackdropSurface` through
Netrender and Vello CPU. The small input files are qualification fixtures.
See [qualification.json](qualification.json) for commits, checks and limits,
and [source-manifest.json](source-manifest.json) for source and artifact hashes.

| Recipe | Source owner | Preserved meaning |
| --- | --- | --- |
| `metre.json` | Knot `SoundReading` | Selected lexical pronunciation, candidate stress, source byte ranges, supplied template and expected rhyme groups |
| `fretboard.json` | Woodshed `StageState` | Real chord catalog, tuning, pitch classes, degrees and independently selectable string/fret occurrences |
| `terrain.json` | Signalman `Survey` | Revalidated original GPX/capture evidence, separate track segments, positioned track samples and unplaced capture identities |

The metre frame includes stress heights and rhyme arcs. Later syllables use
numbered labels rather than invented orthographic splits. Its template also
supports ternary feet. The terrain frame has no coverage tiers: this Survey
provides no admitted radio-position association or propagation estimate, so
the whole plane is marked unmeasured.

## Reproduce

Use the app commits and Mere source manifest recorded in the qualification.
Set `scene_mere` to the Mere checkout and `scene_output` to a new output
directory. The helper patches all four shared scene crates as one cohort.
It can update the app checkout's Cargo.lock; use an isolated checkout for this
pre-integration qualification. Tracked app manifests retain canonical sources.

From each respective app checkout:

```sh
# Knot
python3 "$scene_mere/support/design-studies/backdrops/with_local_scene_source.py" "$scene_mere" \
  run -p knot-readings --features portable-scenes --bin knot-metre-scene -- \
  "$scene_mere/support/design-studies/backdrops/adapters/poem.txt" \
  "$scene_output/metre.json" 1 2 AABB --cmudict

# Woodshed
python3 "$scene_mere/support/design-studies/backdrops/with_local_scene_source.py" "$scene_mere" \
  run -p woodshed-core --features portable-scenes --bin woodshed-fretboard-scene -- \
  C Major 0 5 "$scene_output/fretboard.json"

# Murm / Signalman
python3 "$scene_mere/support/design-studies/backdrops/with_local_scene_source.py" "$scene_mere" \
  run -p signalman --features portable-scenes --bin signalman-survey-scene -- \
  --gpx "$scene_mere/support/design-studies/backdrops/adapters/track.gpx" \
  "$scene_output/terrain.json" \
  "$scene_mere/support/design-studies/backdrops/adapters/capture.json"
```

Then from Mere:

```sh
cargo run -p pictograph --no-default-features --features backdrop-cpu \
  --example backdrop_receipt -- "$scene_output/receipt" "$scene_output"
python3 support/design-studies/backdrops/render_cpu_pngs.py "$scene_output/receipt"
```

The exporter refuses an existing output file. Knot's file source ID records
its canonical input path, so moving the fixture changes source IDs and compiled
generation; compare semantic facts when reproducing in another checkout.
PNG conversion only encodes the actual CPU-rendered RGBA; it does not redraw
the scene. Raw RGBA and compiled JSON are ignored and can be regenerated.

These receipts qualify model adapters, file export, shared compilation, CPU
paint and attention changes. New native app panels, native/GPU/browser
presentation of these exports, release performance and real RF coverage
remain outside this receipt. Earlier Graphshell sample-browser evidence is
preserved separately in `../receipt/qualification.json`.
