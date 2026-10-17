# Pool workspace prototype

The [receipt](receipt.json) binds the small shared `inker::pool` model and Graphshell host to the source/runtime hashes, focused checks and actual browser interaction. Native Knot has its own source-bound receipt. No new crate or canonical consumer repin is implied.

Reproduce with `product,presentation-webgpu`, no default features, and `RUSTFLAGS="-C link-arg=-zstack-size=16777216"`, using rustup 1.99.1. Package the resulting Wasm with `wasm-bindgen --target web`. Serve the package using `ports/graphshell/web/scenario_qualification_server.py` on a fresh loopback origin.

Open `tree.html?app=local&scenario=scenarios/pool.scn&sink=http://127.0.0.1:PORT/scenario-receipt`, then the checked-in `pool_reopen.scn`, `pool_empty_reopen.scn`, `pool_text.scn` and `pool_text_reopen.scn` on the same origin. Each page must finish before navigating. The first scenario expects a desktop viewport with direct portable-scene controls; the compact drawer is a different layout. Use disposable fixture material and an empty Pool for the first run. The text case invokes the normal download action; the receipt does not claim download-byte verification.

All eight final captures were inspected. The prototype utility is usable but expanded Pool plus existing Graphshell chrome crowds the scene stage; final pocket/swatch layout is not accepted by this receipt. Browser origin eviction remains possible and is disclosed by the existing storage inspector. Initial harness/layout attempts remain alongside the accepted runs.
