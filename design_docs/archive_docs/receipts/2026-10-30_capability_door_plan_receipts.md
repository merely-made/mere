# Capability door plan receipts

Run evidence for [the capability door plan](../../mere_docs/implementation_strategy/2026-10-30_capability_door_plan.md),
appended under dated headings (DOC_POLICY §8).

## 2026-10-30: M1, Knot as the first declared application

### Published cohort

| Repository | Commit | What landed |
| --- | --- | --- |
| Armillary | `a333f516a3b8e05c381a3b0b55e617ffe154b592` | `djinn::door::declaration`: `AppDeclaration`, `AppGrant`, `AppAuthority`; declaration on the route hello; `AllowedAppRoutes::{grant_app, serve, permits, authority}`; `AdmittedApp::authority`; `CustodyCall::operation`; `CustodyRefusal::{Undeclared, NotGranted}`; declared client openers |
| Mere | `28a39cef2` | `ports/djinn` renamed `ports/mere-resident` (live references only; receipts keep their paths) |
| Mere | `f278916b8`, lock `94c1bd1a3` | Owner grants (`applications`, `policy/default_applications.json`); Knot out of `CUSTODY_APPS`, the first-party mere list, the published-site grant and `ReleasePolicy::default()`'s epoch rules; declared custody and provider checks; `djinn-site` removed |
| Knot | `3e089ba1` | `declaration::app_declaration()` on every door connection; `knot_site::handoff`; `knot_site` bin (`site-handoff`); Djinn `a333f51`, Mere `94c1bd1a`, Dramatis `6aa0daa4`, Genet `c0144fe5` |
| Mere | lock `52fe900f7`, `4f05821b1` | Knot `3e089ba1` locked; Graphshell personal-sync tests call pictograph's saved-scene readers |
| Cleromancy | `cd6f37a` | Djinn `a333f51`, Mere `4f05821b`, Knot `3e089ba1`; Genet kept on `c0144fe5` |

Between `94c1bd1a3` and `4f05821b1` Mere's main did not compile `mere-resident`
(Knot still locked on Djinn `721e17d`): the two-phase window of the family
repin loop.

### Gates

Local development ran Mere and Knot against local Djinn and Knot through
`--config` patches; the gates below ran from the committed locks.

| Gate | Result |
| --- | --- |
| Djinn `cargo test --all-features` | 81 passed |
| Djinn no-default, Wasm `door`, clippy | Passed; clippy warnings unchanged from `721e17d` |
| Armillary `cargo test --workspace --locked`, Djinn isolated fixtures | Passed; portable cone 17 packages, no forbidden packages |
| `mere-resident` lib | 270 passed, 1 failed (below) |
| `declared_applications` | 3 passed |
| `custody_route`, `vault_lock_doors`, `app_projection_offers`, `custodian_client`, `knot_app_route`, `knot_checkpoint_receiving`, `reservoir_access`, `published_site`, `resident_status`, `keeper_endpoint` | 2, 5, 4, 11, 1, 1, 1, 3, 4, 2 passed |
| `graphshell-endpoint --all-features` | Passed |
| Mere all-targets: `mere-resident`, `graphshell-endpoint`, `graphshell-host`, `graphshell-mcp`, `station-custody`, `mere-overmap`, `graphshell`, `dramatis-app`, `graphshell-mere`; `mere-resident --no-default-features` lib | Passed |
| Knot `-p knot-editor --features site-handoff,projection-provider,peers,mere-overlay --lib --bins --tests` | 20 test binaries passed |
| Knot workspace all-targets | Passed |
| Cleromancy workspace all-targets; `--no-default-features` tests | Check passed; tests passed except one (below) |
| Fresh clones of the published mains | Armillary tests; Knot and Cleromancy all-target checks; Mere `4fcec3ab1` all-target checks above plus `declared_applications`, `custody_route`, `vault_lock_doors`, `app_projection_offers` passed |

Mutation checks: removing `admit_custody` from the resident's custody answer
failed two `declared_applications` tests; removing the declared-offer check
failed the provider test (bounded by a 4-second timeout so a regression fails
rather than hangs).

### Failures and blockers outside this change

- `resident_mere::tests::the_dialable_host_serves_only_what_the_grant_and_the_owner_name`
  fails identically on unmodified Mere `HEAD` before this work.
- Cleromancy `consultation_authoring_dom` fails identically on its prior lock
  ("Authored layout did not close after selecting Four directions").
- Turnstone was not repinned: any Mere relock there fails resolving
  `https://github.com/mark-ik/arboard.git` (HTTP 404, through Genet's
  `genet-clipboard`) and Netrender `main`'s manifest ("invalid type: boolean
  `true`, expected a map"). Its current lock stays self-consistent on Djinn
  `721e17d`.
- `knot-site`'s standalone manifest pins Eidetic `5afe6723`, which no longer
  exists, and its lock is gitignored; it builds only as a dependency.
- Knot's provider factory test compared a temporary path with the
  directory's canonical root and failed on macOS; fixed in Knot `3e089ba1`.

## 2026-10-30: Turnstone and Hocket after the arboard fork's removal

`https://github.com/mark-ik/arboard` (branch `custom-formats`) returns 404, so
Turnstone's lock (`bec2dcd1`) and Hocket's could not resolve even `--locked`.
The "invalid type: boolean `true`, expected a map" message during resolution
is Servo `aac43a3f`'s tidy fixture
`python/tidy/tests/lints-configuration/no-workspace/Cargo.toml`
(`[lints] foo = true`); Cargo 1.97.1 and 1.99.1 print it and continue (a
standalone reproduction exited 0 and wrote its lock). Netrender `main` resolves
on its own.

| Repository | Commit | What landed |
| --- | --- | --- |
| Genet | `b7c0bef0461` | `genet-clipboard` on arboard 3.7; custom formats `Unsupported` on the system backend |
| Woodshed | `e1d93b0d` | Hocket keeps copied audio host-locally; Genet `b7c0bef`; stale `genet-taffy =0.14` patch row removed |
| Turnstone | `554f323f` | Djinn `a333f51`, Mere `845fe1c4`, Knot `3e089ba1`, Genet `b7c0bef`, arboard 3.7.0 (shared with Servo); Murm and Netrender unchanged |

| Gate | Result |
| --- | --- |
| `genet-clipboard` tests | 4 passed, the on-host OS clipboard test ignored (not run; it would overwrite the user's clipboard) |
| Hocket workspace all-targets and tests | Passed; 5 test binaries |
| Turnstone all-targets, default and `authoring` | Passed |
| Turnstone lib tests | 1355 passed, 9 ignored |
| Fresh clones of Turnstone `554f323f` and Woodshed's Hocket | All-target checks passed; neither lock references the fork |
