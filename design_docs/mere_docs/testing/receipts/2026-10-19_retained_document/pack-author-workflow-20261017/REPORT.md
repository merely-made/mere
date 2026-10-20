> Copied 2026-10-19 from the M4-local `~/Code/testing/pack-author-workflow-20261017/REPORT.md`; see the [receipts index](../README.md). Links to these files, which stay local, are shown as plain paths: `reproduction/transcript.json`. Absolute `/Users/markik/...` paths below are host-local reproduction paths, not links.

# Private CLI real pack-authoring workflow

Date: 2026-10-17. Status: passed twice on Isocosm `285821ad6663e75a289ea047769aa9dfb226195c`.
Fetched first and fast-forwarded the clean main checkout from `ecff5762`.
No product code, manifests, schema, admission policy or repository docs changed.
No commit or push was performed. Shared API extraction remains deferred.

## Task and provenance

Prepare a river-heirloom variation of the existing demo generator pack. Copy the
complete real `crates/isometry-system/examples/packs/demo` directory to scratch,
then use the existing annotated manifest fixture as the starting manifest.
The annotation supplies comments and foreign content; scripts and declared
fixture files remain the actual pack assets. Receipts hash every original file,
the annotated starting manifest, final source, runner and built executables.
Original pack hashes and unchanged scratch assets were checked after the run.

The actual private binary is driven interactively over stdin/stdout using only
its documented JSON-line protocol. The driver imports no private Rust modules.
The field edit uses the returned inspection occurrence/value ranges; nested
preset insertion and removal use UTF-8 text ranges in the revision returned by
`read`. The driver uses exact unique text anchors for those latter operations.
This establishes caller feasibility for this fixture, not a general semantic
editing algorithm.

## Observed workflow

1. Read and inspect the annotated pack, then check its four generator choices.
2. Rename `forge_item` to `Forge a river heirloom` through an inspected value
   range. Insert a `quality` preset under that generator and remove its existing
   `culture` preset. Real typed checking confirms exactly the desired preset
   under the first generator and no preset leakage to the other generators.
3. Confirm the display-name comment and complete foreign-extension tail remain.
   Remove the closing quote from the edited generator name. `read` reports an
   invalid revision, `check` refuses it, and explicit `save` preserves the draft.
4. Quit and run the existing loader/fixture executable: it rejects the invalid
   draft. Open a new authoring process and confirm exact invalid source bytes.
   Repair only the missing quote, check, save and quit. Saved bytes equal the
   authored valid snapshot exactly.
5. Run the existing `pack_fixture` executable on the scratch pack with
   `demo:forge_item fixtures/forge_item.json`: it loads and executes successfully.
6. Open another authoring process and insert an unsaved author note. Simulate a
   separate editor changing the disk file. Save refuses with `external manifest
   change; reopen explicitly before saving`; disk retains the external edit,
   and `read` retains the complete pending author draft.
7. Export the pending draft to `pending-conflict-draft.toml` using `read`, then
   quit explicitly. Reopen the external version in a new process, reapply the
   author note, check and save. A fifth fresh process reads exact final bytes,
   reports clean state and a complete index, and checks without admission.
   The existing loader/fixture executable passes again.

Both runs passed. The second run is the reproducibility receipt with runner and
binary hashes: [qualification](reproduction/qualification.json) and
complete command/response transcript (`reproduction/transcript.json`, local only).
It includes five CLI processes and both positive and negative actual loader
observations. The first run and scratch copy are retained separately at this
report's root; all successful outputs and refused responses are preserved.

## Findings and limits

No workflow blocker was found, so no product fix or new semantic command was
added. The public CLI responses were sufficient for a scripted caller to
complete this task. Source offsets still require caller orchestration, and
nested insertion/removal require knowing the intended TOML ownership boundary.
That is usability friction, not a blocker demonstrated by this workflow.

Conflict refusal is useful and recovery is possible with the existing `read`,
quit and reopen operations. The caller must explicitly export a pending draft
before quitting; the tool supplies neither durable session recovery nor an
automatic merge. This qualification demonstrates that explicit recovery path,
not crash safety or prevention of all accidental draft loss.

Checking and saving report `admitted:false`; fixture execution is a separate,
deliberate existing-loader step. No live application session was opened or
mutated. No GUI, arbitrary TOML semantic-edit support, full TOML conformance,
noncooperating-writer race elimination, crash durability, new memory profile or
performance qualification is claimed. No shared API is introduced.

## Reproduction

From `/Users/markik/Code/repos/isocosm` at the recorded baseline:

```sh
cargo build --locked -p isometry-system --features pack-authoring --bin pack-author --example pack_fixture
cargo metadata --locked --no-deps --format-version 1 > /Users/markik/Code/testing/pack-author-workflow-20261017/metadata.json
python3 /Users/markik/Code/testing/pack-author-workflow-20261017/qualify.py /tmp/isocosm-author-workflow-new-run
```

Use an unused output directory; the runner refuses an existing scratch pack.
The driver reads Cargo's target-directory metadata. `build.log` records the
successful build with existing dependency warnings. `reproduction/final.toml`
is the exact final authored source; intermediate valid/invalid drafts and the
exported pending conflict draft are retained alongside it.
