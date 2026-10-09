# Scenograph handoff to Codex: S2, B1, and the dynamics slot

**Date:** 2026-10-09
**Status:** open: handed to the Codex agent by Mark (Scenograph editor plan, SE87).
**From:** the Scenograph editor lane (Claude). **For:** the Codex agent.

Three pieces of ruled work, each with done-conditions in its plan. Read the
plan sections first; they hold Mark's rulings verbatim, and those govern.

## The work, in order

1. **The dynamics slot** (dynamics grammar plan, `mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`, F192 to F195; asked for by the Scenograph plan's SE69).
   - F192: `scenograph::AuthoredProjectionDefinition` gains `dynamics: Option<DynamicsSlot { version, spec }>`, the spec as canonical JSON. scenograph stays on sceno and serde and keeps `Eq`; no seiche dependency. The binding host reads the spec through seiche and refuses by path.
   - F193: the recipe's `arrangement.kind` and the spec's `target.arrangement` must agree; binding refuses a mismatch, naming both paths.
   - F194: `ProjectionVariant` varies dynamics by a catalog preset id or a whole `DynamicsSlot`, one or the other (naming both is refused, a reading not yet ruled: confirm with Mark).
   - F195: SE70's settle is a seiche function, `settle(spec, inputs, bound) -> positions`, every host calls, with a native-against-wasm identity receipt. The default step bound for a spec with no stop rule goes back to Mark as a measured number.
2. **The grid's dynamics axis** (Scenograph plan SE54, SE55, SE70, track S1's last open item). After the slot: `scenograph::swatch::AxisKind` gains `Dynamics`; the projection editor's comparison (`ports/graphshell/src/projection_compare.rs`) can vary it; settled cells call F195's settle; the focused cell runs live (SE55).
3. **B1, backdrops in the Graphshell viewer** (Scenograph plan §2 B1, SE86). Independent of the others.
4. **S2, linked swatches** (Scenograph plan §2 S2, SE84, SE85). First ask the mer3ly site session (`merelyllc.com stack review`) for the sandbox's exact state shapes, which it offered; and put to Mark the new name for appearance-part selection (SE84) before writing it.

## Where things are

- Swatch types: `crates/cambium/scenes/scenograph/src/swatch.rs`. Composition: `crates/cambium/scenes/scenomise/src/facet.rs` (`compose_facet`), tests in `facet_tests.rs`. Subgraph specs: `crates/forme/curation` (published as `mere-curation`).
- The editor's grid: `ports/graphshell/src/projection_compare.rs` and `web_projection.rs` (`compare_targets`, `toggle_projection_compare`, `pick_projection_compare`); headed scenario `ports/graphshell/web/scenarios/projection_compare.scn`.
- The catalog's addition records: `mere_docs/research/2026-08-15_projection_grammar_catalog.md`, "Addition records". Each promoted primitive needs one.

## How this lane works (Mark's standing rules)

- **Forks go to Mark.** Any decision with more than one defensible answer becomes a question with evidence and options, recommendation first. Record his answer verbatim as the next numbered ruling in the owning plan (SE88 onward in the Scenograph plan, F196 onward in the dynamics plan) and commit it the same turn. Mark anything beyond his words *Reading, not ruled*.
- **Evidence before claims.** Check code, counts and paths before writing them down; when evidence contradicts a ruling, reopen it with Mark.
- **Worktrees.** Work in a worktree off `origin/main`; push only your own commits as a fast-forward; never bare `git stash`. Each worktree builds in its own directory (F183): a gitignored `.cargo/config.toml` with `[build] build-dir = "C:/t/cargo-build/mere-<lane>"` and `[profile.dev] debug = 0`. At most three cargo builds at once.
- **Gates.** Tests pass, controls fail where they should, and a gate reruns only when its inputs moved. Headed checks run in Chrome and Firefox: build `ports/graphshell/web` for `wasm32-unknown-unknown`, `wasm-bindgen --target web --out-dir pkg`, serve with `Code/testing/mere/scenograph-editor/sink_server.py <port> <web dir>`, and open `index.html?scenario=scenarios/<name>.scn&sink=http://127.0.0.1:<port>/scenario-receipt`; receipts land in `Code/testing/mere/scenograph-editor/receipts/`. A hidden browser tab gets no frames: keep it visible. Look at the whole screenshot, not only the feature.
- **Docs.** New active docs need a `DOC_README.md` entry and a D2 record (`support/doc-audit/d2/`, checked by `scripts/mere_doc_judgment_audit.py`). Don't reformat whole files that were unformatted on main.
- **Clean up** each lane's worktree, branch and build directory when it lands.
