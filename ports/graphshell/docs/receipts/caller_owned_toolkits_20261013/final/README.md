# Final native supplier qualification

The 2026-10-14 native run qualifies the Mere `eeb59b303fd32611fdab1aea7a52a0f29b004fa0`
working source with the final supplier bridge and adapter metadata instrumentation.
Genet is `b6bfbeb556a6a17eddf8d5ad8254d3fcac95b084`; Netrender is
`6a24ae08f55129b474282b8e4911564f875ce5fd`; Vello remains
`2eae6243881d5b24c984d5ee35dde509ca5db782`. Iced remains
`ab336f6d6403d7206d82c7c2caa46143efde8284` and egui remains 0.36.2.
The normal root and standalone web metadata receipts resolve one final Netrender
four-package family. This run uses the normal root graph, `--locked --offline`,
Rust 1.99.0 and `-j1`, with the existing `C:/t/cargo-targets/mere` target and its
`build` directory. No isolated Cargo home or worktree was created.

From the Mere root, the prepared sequence was executed as
`& .\ports\graphshell\docs\receipts\query_views_20261013\final_native_gates.ps1`.
Its exact Cargo commands, exit codes and raw logs are recorded in the adjacent
`query_views_20261013/final-native-gates.json` and `final-*.log` files.

| Focused gate | Passed |
| --- | ---: |
| Retained mount | 11 |
| Frisket workspace | 7 |
| Native mailbox custody | 1 |
| Visible document polling | 1 |
| Shared query view | 10 |
| Graphshell query owner | 1 |
| Retained clip/export/wheel | 13 |
| Caller-owned embedding/input | 6 |

All 50 focused tests passed, followed by the actual egui callback/Iced primitive
benchmark and the port boundary check. The benchmark produced 32 PNGs, forming
16 egui/Iced pairs. Every pair has identical PNG bytes and RGBA pixels. Every
final PNG also matches its earlier `combined/native` PNG exactly; the earlier
original and combined receipts remain intact. The verified native media body
contains 8044 bytes. The actual Mere graph has 14 nodes and emits 63 operations.
Visual inspection covered the clipped Coffee document and Redshank media view,
the decoded PNG at scale 1.5, and the actual Canvas graph.

The benchmark itself recorded the NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan,
vendor 4318/device 10464, NVIDIA driver 610.88. Both toolkit adapters use the same
single caller-owned Device and Queue. Clipping, scale, independent appearances,
caller submission and explicit disposal are exercised. Resource rehydration
advances generation 1 to 2 on unchanged GPU handles; physical device replacement
or loss was not exercised.

`source-before.json` records 27 source/input hashes before the gates.
`qualify_artifacts.py` checks that those files remain unchanged, requires the
expected test counts and successful exit codes, compares all 32 previous images
and all 16 toolkit pairs, and records source, manifest, lock and metadata hashes.
It was run with Python 3.11.14/Pillow 12.3.0; `qualify-artifacts.log` preserves its
summary. `native/qualification.json` contains the benchmark's measured adapter
and caller invariants. `source-qualification.json` binds those results to the
source and suppliers. `artifact-hashes.json` indexes this final receipt without
duplicating the adjacent gate logs or metadata.

The root Cargo.lock SHA256 is
`77ee8deb58748694b466d3265bc829bab8e3c992326396bf9019f9792d907b89`.
The source qualification SHA256 is
`d78d7beb6f14e1e35c09d98c319834bc6e1ed0341a6e29d6296d1c67de5943b1`;
the runtime qualification SHA256 is
`e2e44bbe989c590303419881198c8192ea50521cc128113750b8f8346b722d9f`.

This qualifies native headless toolkit callbacks through Sparse outline text
and owned CPU images. Classic caller-encoder ownership, glyph-atlas parity,
physical toolkit window input/IME/accessibility, actual device loss and audio
output remain open. Native headed A4 and browser-hosted Graphshell acceptance
have separate owners and receipts. All Cargo/GPU ownership was released before
this CPU-only artifact review.
