# Cycle breaks plan receipts

## 2026-11-06, D7 to D9 / P6 extraction

- Baseline (Mere `49571db83`, `cargo +1.97.1`): `check --workspace --locked` pass; tests for the
  consumers and moved crates: 2,796 passed, 7 failed, 42 ignored. Failures: mere-resident lib
  `the_dialable_host_serves_only_what_the_grant_and_the_owner_name`; `custodian_client` (2);
  `cxf_import` (1); `quarantine_review` `wrong_persona_stale_revision_and_ordinary_app_cannot_mutate`;
  app-host `projection` (2: `values_cross_wasmtime_unchanged`, `the_guest_matches_the_native_transcript`).
- After (Mere `18c0f24a5`, same package set): 2,519 passed in Mere plus 291 in graphshell
  (`--all-features`; 264 default); failures are the mere-resident, custodian_client, cxf_import and
  quarantine_review ones above (app-host projection now passes), and one intermittent failure,
  `graphshell-mere` `connection::tests::browser_failure_is_reported_separately_from_owner_acceptance`
  ("unknown fixture resource"), seen once under load and passing in 7 reruns alone.
- Per-binary diff against the baseline: `frozen` 28 and `shelfmark` 1 new (moved tests), `history` 3
  became `scene_trace_history` 3, `portable_projection` 2 became 1 plus the shelfmark test,
  `wit_copy` 1 removed with the duplicate WIT, `graphshell_client` lib 94 to 72 (22 frozen tests
  moved; 6 AccessKit ones are new in the moved set), `graphshell_door` lib 7 and `graphshell_endpoint`
  lib 21 to 18 (door tests split; feature unification differs).
- Lock roots in Mere: root, `ports/graphshell/web` (1 unused patch, `netrender-vello-shaders`, as
  before), the app-host projection guest and the custody-projection fixtures offer, projection,
  provider, host: relocked, `metadata --locked` twice, no diff. Fixtures: offer, projection, provider
  tests pass; host checks native and wasm32.
- Leftover sweep: zero active references to the old crate paths in manifests, scripts, CI and active
  docs, except frozen evidence: `design_docs/archive_docs`, receipts and logs, the stack ownership
  inventory, `support/design-studies`, `support/family-promotion`, `ports/graphshell/web/fixtures`,
  and stale file lists in `scripts/dr_c_receipts.py`.

## P4 (D2): the ESP adapter leaves Eidetic

- Cycle proof (throwaway script, not committed): `cargo +1.97.1 metadata --format-version 1 --locked
  --all-features` over every lock root of Eidetic (root, `line-ceiling`), Distillery (root, six probe
  roots) and Gemot (root, `address`, `site-host`, `site-model`, `probes/spaces-eval`), resolving
  package edges to repositories (dev edges counted, D6). Distillery to Eidetic 21 package edges,
  Distillery to Gemot 3 (dev), Gemot to Eidetic 54; no Eidetic to Distillery, no Gemot to Distillery,
  no edge in any reverse pair. Result: no two-repository cycle. `eidetic/crates/responsa` was
  skipped: its own lock is stale against its manifest (`--locked` refuses), untouched by P4, holds no
  `esp`, `eidetic` or `distillery` package.
- Before (Eidetic `main` at `2df00e0`, `cargo +1.97.1 test -p eidetic-search --locked`): lib 74 passed
  1 ignored; `lexical_ngram_recall` 2 passed 1 ignored; `mint_ladder` 0 passed 1 ignored;
  `model_free_compare` 6, `model_free_query` 15, `model_free_recall` 11, `model_free_records` 19,
  `recall_mint` 5, `sealed_results` 1. With `--features bert`, `ESP_MINILM_DIR` at
  `Code/testing/eidetic/models/all-MiniLM-L6-v2`, `--include-ignored`: `lexical_ngram_recall` 4
  passed; `goal_model_minilm`: `adapter_end_to_end` ok, `calibrate_match_similarity` FAILED
  (picks 290, constant 220). `--features recall-example --example eidetic-recall`: 4 passed.
- After, ESP (`distillery` `ca9334d`): `-p esp --locked --features goal-model`: lib 93 passed,
  `lexical_ngram_recall` 2 passed 1 ignored, `sparse_dense_parity` 9 passed 1 ignored; with
  `goal-model,bert` and `--include-ignored`: `lexical_ngram_recall` 4 passed, `goal_model_minilm`
  `adapter_end_to_end` ok and `calibrate_match_similarity` FAILED the same way (290 against 220);
  `--features recall-example --example eidetic-recall` 4 passed. `cargo check --workspace --locked
  --all-targets` clean; `-p distillery --features inference-model` tests pass.
- After, Eidetic (`6b4ec49`): `eidetic-search` default tests as in Before minus the moved
  `lexical_ngram_recall` and `goal_model_minilm`; `cargo check --workspace --locked --all-targets`
  clean; lock determinism checked (metadata twice, no diff).
- After, Mere (`b20da96`): `cargo check -p graphshell --features goal-surface-model --all-targets
  --locked` clean, `cargo test -p graphshell --features goal-surface-model --lib goal_surface` 29
  passed 1 ignored; default `-p graphshell --locked` check and `ports/graphshell/web` check clean.

## P5 (D3) and P6 (D4, mere-comms half)

- Cycle proof, P5 (throwaway script, `cargo +1.97.1 metadata --format-version 1 --locked
  --all-features --offline` in every lock root of Gemot (5) and Murm (12), packages resolved to
  repositories, path packages owned by the root's repository, dev edges counted, direct and
  transitive): Gemot to Murm in Gemot `.`, `crates/site-host`, `crates/site-model`,
  `probes/spaces-eval` (from `gemot`, `gemot-site-host`, `commons-spine`, `mien`: `murm-sync`,
  `mere-transport`, `murm`, `murm-resolve`, `retinue`); `crates/address` none. Murm to Gemot: none,
  direct or transitive, in any root. `murm/fuzz` refuses `--locked` (stale lock, no Gemot name in
  it); `testing/webrtc-evaluation` needed the network and was checked from lock text (no Gemot).
- Murm before (`b80cf589`), `cargo +1.97.1 test --locked -p murm-resolve -p murm-holder`:
  `murm-holder` 8 passed, `murm-resolve` 7 passed. After (`c0adc674`): the same, and `--all-features`
  9 and 8. Gemot after (`df6da4a`): `cargo test --locked --features murm-resolve --test
  release_address` 4 passed; `cargo check --locked --all-targets` clean with and without
  `--all-features`.
- P6, history: `git filter-branch -f --prune-empty --index-filter` (index filter mapping
  `crates/shell/comms/` and `crates/graphshell/shell/domain/comms/` to `native/crates/comms/`) on a
  fresh clone at Mere `016927d79`, 7,724 commits rewritten in 451 seconds, 128 kept; `git rev-parse
  main:native/crates/comms` and `016927d79:crates/shell/comms` both `69dbd66fc1cb`. Imported into Murm
  with `git merge --allow-unrelated-histories` (`559470b6`).
- P6 tests: Mere before, `cargo +1.97.1 test --locked -p mere-comms`: 28 passed, 1 doctest; with
  `--all-features` 38 passed, 1 doctest. Murm after, `-p mere-comms`: the same counts. Signalman
  (`-p signalman --features comms`): 75 lib passed, the other binaries pass; `signalman-desktop
  --locked --all-targets` (1.99.1) check clean. Mere after: `-p mere-moot` lib 25 passed;
  `-p graphshell --features workspace-editor --all-targets` check clean, `--lib comms` 3 passed;
  `ports/graphshell/web` `cargo check --locked` clean; `cargo metadata --locked` twice, no lock diff.
  Not run: `mere-moot` tests that load the `guest-settings` guest (its build fails here with exit 101,
  pre-existing, a warning from `app-host`'s build script).
- P6 cycle proof (same script, Mere 17 roots, Murm 12): Murm to Mere direct edges: `signalman` to
  `scenomise` (root); `signalman-desktop` to `cambium`, `cambium-desktop`, `mandrake`, `scenomise`,
  `seiche`, `sprigging`, `tabard`, `tabard-workshop`. Mere to Murm: `mere-comms`, `mere-transport`,
  `murm-sync`, `murm-webrtc`, `retinue`, `murm-ocapn`, `murm-ocapn-transport` in the Mere root,
  `ports/graphshell/web` and `testing/moot-ocapn-coop`. Skipped (`--locked` refuses or needs the network;
  lock text names no `murm` or `mere-comms`): Mere `guest-settings`, `native-id-consumer`,
  `luggage-comparison`, Murm `fuzz`. The cycle stands: P6 is open for F1.

