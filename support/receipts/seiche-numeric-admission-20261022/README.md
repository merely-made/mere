# Canvas runtime numeric admission — 2026-10-22

The Canvas binder now checks the f64 values it narrows to f32: Mix part and Grouped outer weights (including root Schedule stages), target anchored pull, and damping. Finite overflow and nonzero underflow are refused at the authored field path before live state changes. Explicit zero and representable subnormals remain valid. The canonical record retains its authored f64 values. Typed Mix/Grouped actions use the same binding authority; their separate guard was removed.

The red regression observed direct binding accept `1e300` as an infinite Mix weight. Final gates pass: 22 core binding/controls tests and 332 Graphshell Web library tests, with six existing ignored tests. Host tests pass malformed values through actual v4 codicil open, asserting unchanged graph, session, Canvas record/geometry and saved facet; save/reopen accepts zero, minimum f32 subnormal and exact authored `0.1` for each supported placement.

`gates.json`, compressed logs, `source.json` and `qualification.json` record the source and validation boundary. Rust 1.99.1, debug0, incremental0, four jobs, locked/offline gates. A locked fetch filled the missing upstream Knot revision; manifests and locks are unchanged. Full local evidence: `/Users/markik/Code/output/mere-seiche-three-passes-20261022/numeric`.

This pass adds no schema, runner, raw-term execution, checkpoint restoration or render dependency changes. Seiche keeps its general f64 admission; the Canvas owns its runtime representation limits. The numeric preflight visits supported flat roots and immediate Schedule stages only, leaving depth and unsupported-shape checks to existing derive.
