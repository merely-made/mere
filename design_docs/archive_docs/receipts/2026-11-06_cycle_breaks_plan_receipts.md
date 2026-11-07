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
