# One-version plan receipts

## 2026-11-17: unused kernel accessibility adapter

Review candidate based on Mere `9b96cd693846c38212a10498ccb78ec733d2995a`.
Consumer: Graphshell `c53ded07115d3857c62170f42af872d47a8b6e34`.
Authority: Graphshell uxtree-spine D8 and P8; one-version step 1.

Raw output and the baseline comparison script live under
`Code/testing/engine-split/one_version_enumn/`; `report.md` is the reader.
Receipt budget: 100 MB across repositories for this lane.

The baseline native and wasm inverse trees show
`registry+https://github.com/rust-lang/crates.io-index#enumn@0.1.14` held only by
`registry+https://github.com/rust-lang/crates.io-index#accesskit@0.25.2`, held only
by `mere-kernel`. After trees and metadata contain neither package. The baseline
grep finds `descriptor_for` and `AccessibilityDescriptor`; the edited source
grep finds neither. The preserved `SurfaceId` source compares identically,
including serde derives and variant order, after newline normalization.

Commands, using stable targets `C:/t/cargo-targets/mere` and
`C:/t/cargo-targets/graphshell`, with `CARGO_BUILD_BUILD_DIR` set to each target's
`build` directory:

```text
cargo tree --workspace --all-features --target <native-or-wasm> -i enumn --format '{p}'
cargo tree --offline --workspace --all-features --target <native-or-wasm> --format '{p}'
cargo tree --offline --workspace --all-features --target <native-or-wasm> -d --format '{p}'
cargo metadata --offline --format-version 1
cargo test --workspace --all-features -j 4
cargo check --offline --workspace --lib --target wasm32-unknown-unknown -j 4
cargo test --manifest-path host/Cargo.toml -p ux-events -j 4 --config <local-kernel-patch>
cargo check --offline --manifest-path host/Cargo.toml -p ux-events --all-targets --target wasm32-unknown-unknown -j 4 --config <local-kernel-patch>
```

Native means `x86_64-pc-windows-msvc`; wasm means `wasm32-unknown-unknown`.
The consumer-only CLI patch selects this worktree's `mere-kernel` from the
`https://github.com/merely-made/mere.git` source; no manifest patch is committed.

- Native workspace: 1,035 passed, zero failures, seven existing ignores across
  32 test/doc-test groups. Three tests of the removed adapter leave with it.
- Graphshell `ux-events`: 43 passed, zero failures; all targets compile for wasm.
- Default Mere workspace libraries: wasm check passes. This is compilation,
  not browser execution.
- Broader wasm checks remain separate: kernel tests import native-only RDF
  apply helpers; enabling `pandect/personal-sync` reaches four non-`Send` stream
  errors in Eidetic's `stickleback/src/store/logs.rs`. Both reproduce on the
  unchanged baseline: the kernel's 314 lib-test diagnostics match by message
  and source location, and both all-feature library checks fail at the four
  Stickleback streams. Failed output is retained. The initial comparison also
  counted concurrently failing examples; the final comparison isolates the
  kernel's complete lib-test diagnostics.
- Normal workspace trees retain five direct Syn 2 holders on native and four
  on wasm; removing `enumn` does not clear the Syn 2/3 split.

Primary Mere and Graphshell checkouts have unrelated WIP. The Mere candidate
worktree stays for review; the temporary Graphshell consumer worktree is
removed after its checks. No isolated Cargo home or target is needed.
