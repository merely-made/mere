# Retained content through Pelt and Genet

Status: qualified bounded continuation, 2026-10-13.

Graphshell presents owner-held HTML, selected Fleece text and verified WAV clips
through Pelt in its existing Frisket Workbench. Source identity, admission,
custody and link decisions stay with the application. The browser fixture uses
an explicitly named IndexedDB store and actual Fleece import/export operations.
Reopening reads that store without reseeding it.

Turnstone `d5d4737` adopts published Mere
`0d1dc1927859ff1aa2a43dd88a04391bc1086fd8`, after the A4/S3 outer frame and
`7d62a5a` Tabard/Interface theming draft. Final consumer qualification uses normal
locked git sources, one Mere source and one Genet revision. Preliminary local
source patches are retained as distinct evidence, not the final pin receipt.

## Checks

All Cargo commands use `--locked --offline`. Root native commands run from Mere
with `CARGO_TARGET_DIR=C:\t\cargo-targets\mere`; the wasm build runs from
`ports/graphshell/web` so its committed target configuration applies. Turnstone
uses its stable `C:\t\cargo-targets\turnstone` target.

| Evidence | Result |
|---|---|
| `final-content-tests.log` | 9 retained-content tests, including actual host pointer routing and clip play/stop effects |
| `document-canvas-tests.log` | 85 document-canvas tests |
| `shared-clip-tests.log` | 2 shared clip tests |
| `merged-resource-tests.log` | 31 unit and 11 owner resource tests |
| `merged-moot-mount-tests.log` | 8 admitted native resource-mount tests |
| `shared-web-host-tests.log` | 12 shared browser mirror tests |
| `final-web-build.log` | Graphshell wasm build passes |
| `turnstone-pinned-check.log` | Windows check against published dependencies passes |
| `turnstone-pinned-clip-tests.log` | 10 Turnstone clip tests |
| `turnstone-pinned-store-tests.log` | 5 Turnstone pane-store tests |
| `turnstone-supplier-closure.json` | One Mere source, one Genet revision, no duplicate Genet package names |
| `final-port-boundaries.log` | Port and browser dependency boundaries pass |
| `final-doc-audit.json` | Active index/status/link/path checks pass; inherited header findings remain recorded |

After merging the upstream viewer lane at `bf07a5a6`, the default application
wasm build and all 9 retained-content tests pass again. The root test command
now selects `--no-default-features --features retained-content,image-decode`.
`post-integration-*` logs record those gates and refreshed dependency/doc audits.
The retained runtime, reading view and browser-host repair source is unchanged
by this merge. The earlier browser captures retain their exact source/artifact
qualification; they are not relabeled as a fresh browser run of the merge.

The browser receipts exercise actual rendered glyphs and registered producers,
then reopening, advertised actions, autoplay refusal and Stop. Browser gesture
observations are separate JSON/image evidence. A resolved `play()` promise proves
browser playback acceptance; it does not prove physical audibility.
`voice-pointer-started.json` and `voice-pointer-stopped.json` record successful
pointer activation after the host frame settles. `voice-gesture-started.json`
records keyboard activation. The second keyboard attempt retained the start
status and is preserved as `voice-keyboard-second-activation.json`; it is not
Stop evidence. The final browser run records no console errors.

## Repairs and evidence limits

The retained view stops its click from activating and replacing an enclosing
tile before pointer capture begins. Shared mirror focus uses a non-scrolling
clip box and a reentrant focus listener guarded by the existing movement flag.
The source supplies exact host font bytes, checked producer keys and real paint
box dimensions. Earlier blank-producer, missing-font, cramped-pane and mutable
fill captures are preserved under named rejected-attempt directories.

The restored ambient-file fallback fails its negative control as expected; the
fallback was removed and the final guard tests pass. The earlier incomplete
replacement control and cancelled/wrong-directory build attempts are not
acceptance evidence. Browser file-chooser upload could not be qualified because
the extension lacked file-URL access; the core receiving/admission path is tested.

The [clip export continuation](clip-export-browser/README.md) adds the host
PortableClip adapter, exact native custody controls and a trusted browser handoff.
It keeps completed download-file verification separate and open. The original
captures above retain their earlier source basis. Protected personal custody,
arbitrary media/engine adapters, physical assistive technology and cross-platform
headed acceptance remain open. Turnstone A4/S3's subsequent implementation and
native qualification are recorded in its stations/windows receipt; this earlier
receipt does not close the whole A4 plan.

Generated browser-package hashes and cleanup are recorded in `qualification.json`.
The package and loopback server are temporary; receipts and ordinary stable Cargo
targets are retained. No isolated target, Cargo home or worktree was created.
