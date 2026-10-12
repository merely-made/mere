# Browser preparation recipe — preserved historical preparation

Root subsequently executed and qualified the [current browser gate](README.md).
The preparation agent did not launch the runtime; the statements below describe
its preparation-stage work.

This lane uses `ports/graphshell/web/tree.html?app=local`, the production
Rootstock CanvasProducer, the existing Taproot scenario runner, and the actual
Genet web-host composed-frame readback. It preserves the complete application
tree. The new scenario exercises cold/repeat painting, private rule drafting
and discard, selection, keyboard camera routing, details, and zoom while
asserting graph/session/source authority. The driver additionally exercises
four application appearance modes, narrow/wide resize, and fresh-page rebind
under the same isolated origin and context.

No browser, adapter, server, build, or qualification has been launched by this
agent. Root owns the serialized two-job build and all GPU/browser execution.

## Artifact reuse audit

- Current Mere source: `ec1e65caed335010308566c8db51ad0e1e002799` before this
  new scenario; current standalone web pins Netrender `ef5412d760a978584c8c1801e8a765c51aecd403`,
  Genet `4bc75583865c2fb862b7acd7fe16dc28eb591042`, Classic provider
  `5862bba26c8a9f56f6a5fdc8ea6966b360a5d63a`.
- The recent `forme_feedback_field_browser_20261011` Chrome receipt qualifies
  Rust `eb804d07` and scenarios `eac1524f`. It already used the same Classic
  provider, but Netrender `46606610` / Genet `cec760f4`. Current `web_tree.rs`
  and `web_tree/lane.rs` differ from its exact SHA-256 records. Current source
  also adds presentation binding/editor/realization, Rootstock capture, and
  Canvas dynamics changes. Its newer-integration section explicitly says the
  newer source was not rerun; its generated Windows browser package and
  stripped input were removed, with shared Cargo target retained.
- `presentation_live_20261011/qualification.json` records a passed product
  Wasm check, **not a build**, and explicitly says default viewer release was
  not built and interactive qualification remained pending.
- Local `output/graphshell-viewer-commands-20261011/web/pkg` (223 MiB Wasm)
  binds older `520b041f` source. Local `output/tabard-graphshell-fixed-renderer-20261010/web/pkg`
  (418 MiB) binds older renderer `491c376c` / Genet `7422e906`. Neither is a
  current-source artifact. Existing Cargo caches may reduce compilation; an
  older artifact cannot replace the source-bound current build.
- Root paused builds at 195 MiB free. The cache owners retired only their own
  disposable incremental state, restoring about 10 GiB, and Moot leased its
  idle `mere-s2s3` target exclusively for this batch. Borrowed caches are retained.

## Root execution recipe

Paths below are explicit; choose another isolated free loopback port if needed.
Keep the existing shared default package directory untouched. The target path
should be a suitable existing root-owned Cargo target, after checking capacity.
Do not rebuild or package while another GPU/browser lane is executing.

```sh
REPO=/Users/markik/Code/worktrees/mere-vello-release-adoption
LANE=/Users/markik/Code/output/vello-classic-browser-gates-20261011/browser
QUAL_TARGET=/Users/markik/Code/targets/mere-s2s3
python3 "$LANE/identity.py" preflight --repo "$REPO" --output "$LANE/source-before.json"
```

If root uses local Cargo patch configuration, include each external source root
in **both** identity invocations with `--extra-source PATH`. Seal refuses any
local metadata package outside those hashed roots. Qualified Git pins need no
extra path. Build and metadata must use the same Cargo configuration/features.

```sh
CARGO_TARGET_DIR="$QUAL_TARGET" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
  cargo build --locked --manifest-path "$REPO/ports/graphshell/web/Cargo.toml" \
  --target wasm32-unknown-unknown --no-default-features --features product -j2 \
  > "$LANE/product-build.log" 2>&1
cargo metadata --locked --manifest-path "$REPO/ports/graphshell/web/Cargo.toml" \
  --format-version 1 --no-default-features --features product \
  > "$LANE/product-metadata.json"
wasm-bindgen "$QUAL_TARGET/wasm32-unknown-unknown/debug/graphshell_web.wasm" \
  --target web --out-name graphshell_web --out-dir "$LANE/pkg"
python3 "$LANE/identity.py" seal --repo "$REPO" \
  --preflight "$LANE/source-before.json" --package "$LANE/pkg" \
  --metadata "$LANE/product-metadata.json" --build-log "$LANE/product-build.log" \
  --output "$LANE/identity.json"
```

Use wasm-bindgen matching the resolved `wasm-bindgen` crate (current source
family `0.2.129`). These commands select the actual product tree while leaving
the optional resident GPU-physics lane out of this renderer gate.

Start the existing isolated-package server in a separate root-owned session:

```sh
python3 "$REPO/ports/graphshell/web/scenario_qualification_server.py" \
  --package "$LANE/pkg" --output "$LANE/server-receipts" --port 8897
node "$LANE/qualify.cjs" --base http://127.0.0.1:8897 \
  --output "$LANE/run-1" --identity "$LANE/identity.json" \
  --scenario "$REPO/ports/graphshell/web/scenarios/renderer_acceptance.scn"
```

The driver launches an ephemeral Chrome context, with no unsafe-WebGPU,
software-renderer or GPU-disabling flags. To use root's explicitly handed-off
connected Chrome instead, add `--cdp URL`; the driver creates a new context and
does not close the connected browser. It closes its owned context at completion.

The existing capability page requests the current default adapter. Missing
secure context/WebGPU/adapter hard-refuses. The web manifest enables WebGPU,
with no WebGL fallback. The probe does not currently expose fallback adapter
classification, so the final receipt discloses that limitation.

Each stage writes its result JSON, actual GPU readback PNGs, screenshot, source
identity and nonblank pixel statistics. Pixel metrics check real decoded frame
pixels rather than trusting `RESULT ok` alone. Original PNGs still require root
visual inspection; entropy/nonblank guards do not prove cross-renderer parity.

## Source-bundle boundary

Browser CanvasProducer implements the GPU producer contract and currently
inherits `portable_scene`'s explicit Unsupported refusal. This lane verifies its
real current production source through GPU execution and readback; it does not
export an intact portable producer-resource bundle or prove CPU/Classic parity.
A proposed additional seam would retain the exact Scene already produced by
the live callback, bind its identity/generation/geometry, and refuse dirty,
unrendered, remote or mismatched snapshots. It must not run another physics frame,
reconstruct a DOM, drop commands, or invent retained-fragment resources. That
source change and a browser-side bundle export trigger await root coordination.
