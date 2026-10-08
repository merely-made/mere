# Genet compatibility

Cambium consumes Genet through published seam packages. `cambium-nematic`
adds reactive projections over Errand's portable smolweb ASTs. Genet's
engine-side Livery and Scripted document runtime remains in `genet-documents`;
Mere's application-facing reader and smolweb lanes live in
`mere-document-lanes`.

**Corrected 2026-10-06:** `cambium-nematic` folded into `cambium` as feature
`nematic` on 2026-09-25, and that feature and its views were removed on
2026-10-06 under ruling S66 of the Mere stack seams plan (the
[smolweb fidelity plan](../../nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md)
WS4, R0): smolweb renders through `mere-document-lanes` only. The published
`cambium-nematic` 0.3.1 stays on crates.io; its table row is history.

## Historical verified Genet seam set (2026-07-22)

Verified on 2026-07-22:

- `genet-scripted-dom = 0.1.0`
- `layout-dom-api = 0.1.0`
- `errand = 0.1.3`
- core provider release commit:
  `2e462fe8975`

| Package | Version | Current source |
| --- | --- | --- |
| `layout-dom-api` | 0.1.0 | crates.io and sibling path |
| `errand` | 0.1.3 | crates.io and sibling path |
| `genet-paint-types` | 0.1.0 | crates.io |
| `engine-observables-api` | 0.1.1 | crates.io |
| `genet-static-dom` | 0.1.0 | crates.io |
| `genet-scripted-dom` | 0.1.0 | crates.io and sibling path |

## Historical Cambium stack: source vs registry (updated 2026-08-20)

| Package | Workspace | crates.io | State |
| --- | --- | --- | --- |
| `meristem` | 0.2.0 | 0.1.1 | 0.2.0 pending publication; breaking scope cut |
| `sprigging` | 0.2.1 | 0.2.1 | current |
| `cambium` | 0.3.3 | 0.3.3 | current (published 2026-08-09; 0.3.2 lacked the IME composition surface) |
| `cambium-nematic` | 0.3.1 | 0.3.1 | current |
| `cambium-winit` | 0.3.0 | 0.3.0 | current (published 2026-08-09; 0.2.0 remains yanked) |
| `cambium-winit-a11y` | 0.3.0 | never published | `publish = false` by design |

The registry graph resolves as of 2026-08-09. The history, kept because it
explains the yank and the split: `cambium-winit` 0.3.0 became publishable on
2026-07-26, when the accessibility host moved out to `cambium-winit-a11y`
exactly because `genet-layout` and `genet-winit-host` inherit Genet's
`publish = false`. Publishing it then surfaced a second defect: the local
`cambium` had grown the IME composition surface after 0.3.2 shipped, so
package verification failed against the registry; `cambium` 0.3.3 publishes
that state under its own number and `cambium-winit` 0.3.0 requires it.
`cambium-winit-a11y` can never publish, and that is the reason it exists:
holding the genet-coupled half keeps `cambium-winit` down to `cambium` +
`winit`.

At that table's date, consumers followed a git-first family rule through
`genet.git`; the registry served external consumers. Cambium Nematic's release
boundary was Cambium plus the protocol AST package, without Genet's layout or
rendering engine.

## Current Mere-owned compatibility

### 2026-10-07 Forms field ownership migration

**Status, 2026-10-08:** the permanent Genet Forms repin at Mere `632c1d29`
qualifies against public Git sources on ThinkPad: 362 package passes, 63
native-host passes and a Wasm accessibility-example compile, with zero failures
and two existing ignored doctests. F6's accessible leaves are included. The
older F5 public-pin sibling gates pass; their refreshed integration gates and
publication are tracked separately in the Forms plan.
The implementation plan and numbered rulings live in Genet's
[dated Forms plan](https://github.com/merely-made/genet/blob/main/design_docs/2026-10-07_forms_value_validation_submission_plan.md).
F3 authorizes this Mere migration and verified repin. The current modern Genet
family pins published `e84f9c7f9aec23320c539784961d1465f8a53a9b`; published
Knot retains its two explicit Genet `965b64e2` identities. The dated receipts
below retain their original source boundaries.

F6 preserves the app textbox's identity, existing name, committed value,
geometry, state, actions and focus while excluding its decorative accessibility
descendants in the neutral projection and the native AccessKit adapter. The
browser mirror inherits that neutral topology. Drawing children stay in the
DOM. F5 limits sibling edits to selectors, caret routing and existing labels in
Turnstone and Cleromancy; unnamed fields stay unnamed. The numbered rulings in
the Forms plan own these decisions; their qualified local gates are recorded below.

Native HTML inputs and textareas now keep live values in Genet's form-control
arena. Cambium's `TextInput` owns app editing and paints highlighted committed
runs, IME preedit, ghosts and carets as children. Those fields therefore use
ordinary `div` elements with `role=textbox`, rather than native HTML form tags.
Multiline fields carry `aria-multiline=true`; public field view APIs stay the
same. Existing field CSS, caret lookup and semantic tests follow the app marker.
Known visible labels are attached explicitly to the textbox, since a `div`
cannot inherit a native input's wrapping-label association.

`data-cambium-text-value` carries only `TextInput::text()`. Rootstock's existing
neutral `document_projection` and the native AccessKit adapter read that
Mere-owned marker on explicit textboxes. They preserve node identity, labels,
geometry, focus and actions while exposing committed value and multiline/editable
state. Preedit, ghost and caret children remain in drawing and are excluded from
the accessible value. Genet receives no Cambium marker contract. Text-valued
accessibility SetValue routing remains a separate existing gap.

Qualification uses the focused Cambium, Rootstock, native adapter and native
host package tests, including single-line editing, IME/caret routing, Unicode
committed-value projection, catalog behavior and the browser accessibility
example. A local consumer run must identify the exact candidate Genet source and
lock; the later published-source repin must separately verify coherent family
identities and repeat the affected gates. Human assistive-technology acceptance
and standalone browser-hosted operation are not inferred from unit tests.

Preparation against committed Genet `b8a3ec1d6abe88e07438ca4d53b9ca4d2b92111d`
preserves all 1,688 locked package versions, dependency arrays and checksums.
Only 33 current Genet, nine Boa and three Vano source revisions move; the two
legacy Knot-owned Genet identities stay unchanged. The default resolved graph
does not activate either JavaScript engine. The candidate lock is frozen as
`testing/genet/forms/mere-forms-lock-preparation.Cargo.lock` under the Code root.
The catalog acceptance program passes and regenerates both HTML receipts with
the app textboxes and their existing names; this is artifact preparation, with
fresh consumer tests still pending. Each run restores the original manifest and
lock and checks all tracked source bytes and mtimes outside its owned outputs.
The optional Cambium `highlight` feature also needs its own library test gate.

The first four-package consumer build fails before tests: Rootstock's
`MutationRouter` exhaustively matches the old mutation enum. The compatibility
repair routes `FormControlStateChanged` by its node to the owning window and
forwards arena state through `WindowDom`, preserving the root's document view.
Two fixtures cover live value versus default attribute and settled layouts in
two windows. Independent controls will disable routing and forwarding before a
fresh restored run. These changes require the candidate seam and are not claimed
to compile against the retained starting Genet pin; publication must carry the
verified repin with this migration.

The owned-snapshot correction qualifies four packages: Cambium 252, Rootstock
72, native accessibility 22 and browser mirror nine passing tests, with zero
failures and two existing ignored doctests. The native routing target then
finds a missing trait import in its migrated fixture before tests; that import
is corrected for the next gate. Catalog and highlight qualification and the
independent negative controls remain pending.

The mirror review finds a separate accessible-tree fork. Genet projects
descendant elements beneath app DIV textboxes; Rootstock decorates only the
parent's committed value, and the browser mirror lowers both that value and
the projected children. Current full-pipeline tests use an empty field and
check its name/role, while decorated-value fixtures stop at the neutral/native
adapters. Mark is asked whether to expose app textboxes as accessible leaves
across adapters or prune children only in the browser mirror. No pruning policy
is implemented before that ruling; drawn children stay app-owned in either case.

The focused native gate executes 59 passes and one failure before stopping:
an unsized app field shrinks to its child text (87px for short text, 400px for
long text), whereas the former native input has a value-independent 20-column
width. The repair gives single-line fields a font-relative substitute intrinsic
width through `contain-intrinsic-size`, preserving Genet's existing half-em
column metric. `--cambium-field-intrinsic-width` lets the host override that
fallback, and explicit host CSS widths retain priority. The new fallback and
its host override require fresh native, component and catalog gates.

The attempted substitute does not qualify: the fresh six-target native run
executes 61 passes and two failures, with the same 87px short field and 400px
long field. Sources are stable and the starting manifest/lock are restored
exactly. The host harness now exposes its existing live computed-style reader
so these regressions report resolved sizing properties before any further
layout repair is chosen. The width contract remains open.

The diagnostic run confirms the live host resolves the requested containment
and substitute sizes correctly, including the host's 12em override; six other
field cases pass. The remaining width failure reaches Genet's CSS layout
handoff, so a bounded repair awaits an explicit checkpoint. Independent
Rootstock native-state bridge controls proceed while that ruling, accessible
leaf policy and sibling scope are pending without response deadlines.

Mark approves the bounded Genet layout repair as F4 in the dated Forms plan.
Mere's width contract remains unqualified until its repaired candidate passes
fresh consumer gates. The other two questions remain pending.

The first disabled-routing attempt executes 71 passes and the intended single
failure, but is not qualified: this note changed during its guarded run, and
the end-of-run process scan could not inspect a compiler that then exited.
The starting manifest, lock and router are restored. Preserve that attempt;
a committed-source retry must establish the control and a fresh restored pass.

The bounded Genet repair is qualified at `e84f9c7f`: 841 passes across Livery,
documents, render and WPT, with zero failures and nine existing ignores. Its
substitute route retains Block/Leaf root admission. Mere's frozen revised lock
changes 33 source revisions and 31 dependency source references without moving
versions, graph shape/checksums or the two legacy Knot identities. Locked
metadata resolution and actual native-field tests remain required here.

Locked resolution now verifies the same 1,564-node default graph after source
revision and checkout-path normalization. It resolves 28 of 33 locked current
Genet packages, preserves both legacy identities and activates neither engine.
The uncached Git revision is fetched from the primary checkout using only the
process-local rewrite. All six native targets then pass 63 tests, including
the repaired default and configurable widths and existing caret/scroll/routing
behavior. Exact manifest/lock restoration and unowned source bytes/mtimes are
verified. Catalog and bridge-control qualification follow; the two outstanding
policy/scope questions still await Mark's answers.

Fresh catalog generation at the repaired candidate changes only the two
single-line textbox styles in each HTML receipt, adding the configured
containment substitute. Its guarded transaction preserves all other tracked
source bytes and mtimes and restores the starting manifest and lock. Generation
is not a catalog test pass. A read-only multiline review finds no newline or
multiline-state loss through neutral, native or browser lowering; the existing
fixtures cover those seams separately, rather than as one full-pipeline case.

The fresh native bridge controls each produce 71 passes and exactly their
intended failure when routing or forwarding is disabled. A fresh restored
Rootstock library run passes all 72 tests. Catalog acceptance passes both tests,
the optional highlighting library passes 254, and the browser accessibility
example compiles for `wasm32-unknown-unknown` with the existing getrandom setting.
All use the frozen `e84f9c7f` candidate lock and restore the starting inputs.
The routing receipt also has an independent post-restoration source audit;
later runs perform that second scan inside the guarded helper. These are local
automated receipts, not browser-hosted operation or human AT acceptance. The
accessible-leaf and sibling-scope questions remain pending.

The final restored four-package gate at Mere `ee699000` passes 355 tests:
Cambium 252, Rootstock 72, native accessibility 22 and browser mirror nine,
with zero failures and two existing ignored doctests. The disabled committed-
value control has exactly two intended failures and 74 other passes. Its first
attempt is retained as unqualified because the helper expected the native test
under `tests` rather than its actual `dpi_tests` module; the corrected fresh
run validates, restores both producers with fresh mtimes, and precedes the
restored pass. All guarded inputs remain unchanged outside owned restoration.

Genet's fresh optimized runner passes all 15 value-model fixtures on both
engines and its CSS guard reports `unexpected=0`; all sixteen summary counts
match the starting guard and expectation files remain unchanged. The qualified
product stays `e84f9c7f`; its frozen runner head is the documentation descendant
`79a7f011`. This completed the gates authorized before F5 and F6; their
implementation receipts follow below before final acceptance and the verified repin.

The downstream audit found old tag checks in Turnstone and Cleromancy. F5 now
authorizes their mechanical selector/caret/existing-label edits, committed in
Turnstone `c951afa` and Cleromancy `523d54d` plus fixture correction `b6b521d`.
Turnstone's corrected public-pin library passes 674 tests with zero failures
and nine existing ignores. Cleromancy's `85c8f77` passes 12 library and three
DOM checks with zero failures or ignores; its independent routing control
produces 11 passes and exactly the intended marked-DIV failure.
Consumers do not stamp
Mere's marker or duplicate its committed-value projection; unnamed fields keep
that state. Existing native-tag support preserves the current public pins.

F6 is implemented and locally qualified at Mere `7d133ddc`. Neutral and native
projections prune app textbox descendants; the browser mirror consumes the
neutral topology. Native pruning follows producer decoration and removes hidden
action routes. Nested descendant focus resolves to the surviving outer textbox.
The drawing DOM remains intact. Unicode/newline fixtures, nested native focus
and the full neutral-to-browser lowering cover these seams. Generated pseudo
rows are traversed, without a separate pseudo fixture in this slice.

`mere-forms-accessible-leaves-disabled` preserves committed-value decoration
but disables pruning: 83 passes and exactly four intended failures across the
neutral, native and browser libraries. The first restored run retains a Windows
DLL-initialization launch failure before native tests (`0xc0000142`), so it is
unqualified. The unchanged restored retry passes 357 tests: Cambium 252,
Rootstock 72, native accessibility 23 and browser mirror 10, with zero failures
and two existing ignored doctests. The fresh Wasm accessibility-example compile
also passes. Guards verify unchanged unowned source bytes/mtimes and restore
the starting manifest/lock. The same frozen `e84f9c7f` lock is used throughout.
No published-source repin or human AT/browser operation is inferred.

Published main `45f5a80c` is integrated locally at `2a89d8dc`, preserving its
command/edit-history work. Fresh starting/candidate metadata qualify the
combined graph: 1,566 packages, equal versions, dependency definitions, edges
and features after authorized revision/checkout normalization, both legacy
Knot identities retained and neither JavaScript engine active. The integrated
1,690-row lock SHA256 is
`06BCF3765C5FDA5282F24C4D675A919E297A3587AA4D9BFFCB0CB928DB4D1EF9`.
The three leaf production/fixture files match the accepted negative control
byte-for-byte. Fresh integrated positive gates pass Cambium 257, Rootstock 72,
native accessibility 23 and browser mirror 10, with zero failures and two
existing ignored doctests. All six native host targets pass 63 tests, and the
Wasm accessibility example compiles. Guarded transactions preserve unowned
bytes/mtimes and restore the live starting manifest/lock. The then-qualified
repin patch is `Code/testing/genet/forms/mere-forms-integrated-repin.patch`;
the later published Knot integration below supersedes this proposal.
Publication and verification against the published Git source remain pending.
Shared unpublished main also contains four other-owner Vault/Lattice
documentation commits; a normal main push would carry them.

Published Mere `f67f5080` is now integrated at `9105b1ef`, preserving Pelt,
dataset and Knot-owner work. Its Knot `eabd4434` repin changes the source
boundary: the starting graph has 1,564 packages; the Forms candidate has 1,566,
with separate protected Fleece 0.5.0 and LayoutDom 0.1.1 at Knot's explicit
Genet `965b64e2`. Mere's modern 33-package Genet family moves to `e84f9c7f`.
Versions, package definitions, modern edges and features are preserved after
revision mapping. Knot's edges remain at its published revision; LayoutDom's
capture feature stays on the modern identity and neither JavaScript engine
activates. The candidate's 1,690-row lock SHA256 is
`3537E6067130E1D8993DAC7DB2F143CB9AA60099879F20EBD654DAE79CA97735`.

Fresh current-source gates pass Cambium 257, Rootstock 72, native accessibility
23 and browser mirror 10, with zero failures and two existing ignored doctests;
all six native-host targets pass 63 tests and the Wasm accessibility example
compiles. The first two-thread package run stalls after three GPU cases; only
its owned test executable is terminated. Its guards restore all source/config/
lock inputs, but the attempt remains unqualified. The unchanged full suite
passes with one test thread, including every GPU case. Leaf production/fixture
hashes still match the accepted four-failure negative control. Current receipts
are `mere-forms-current-knot-packages-serial-retry`,
`mere-forms-current-knot-native`, and `mere-forms-current-knot-wasm-a11y` under
`Code/testing/genet/forms`. All guards preserve unowned bytes/mtimes and restore
the starting inputs. The review patch
`mere-forms-current-knot-repin-review.patch` passes `git apply --check`; its
review-proposal JSON records the exact candidate manifest/lock byte hashes.
The earlier raw-checkout patch and integration receipts remain historical.
Publication and public-source verification still precede a permanent repin.

Cleromancy's older public `OnKey` has no `.attr` method. Its bounded local
`NamedText` view attaches only an existing visible name to Mere's produced node
and forwards the field lifecycle/messages. It adds a direct import of
already-locked Meristem 0.2.0 from the same Mere `8106c7c` family. The 1,005 lock
packages retain their versions, sources and checksums; all 668 resolved package
identities/features remain, with only the direct root edge added. Both public
families remain unchanged: Mere `8106c7c`, Genet `34626a6c`. The final generic
DOM setter avoids a test-only node import. Interim compile failures remain
unqualified evidence. These checks prepare the sibling for app fields; family
adoption remains a separate gate.

Existing target `C:/t/cargo-targets/mere` is reused without another live owner;
Genet's qualification retains its borrowed `C:/t/cargo-targets/genet-encoding`
and `C:/t/cargo-homes/genet-streams` for the recorded gate owner. No new isolated
target, Cargo home or worktree is created in this slice. The helper's generated
bytecode under `Code/testing/genet/forms/__pycache__` was removed on 2026-10-08
after Mark explicitly authorized cleanup. Its earlier automatic-review
rejection ("blocked by policy") remains historical. Mark also authorized
publication and push, and requested remaining serial tests on ThinkPad.

### 2026-10-08 public-source Forms qualification on ThinkPad

Mere `632c1d298dee1ecd5c3dfe00879c95d4a012f8ea` adopts Genet
`e84f9c7f9aec23320c539784961d1465f8a53a9b` after integrating published Mere
`4fd2f3f1`. The permanent manifest and lock preserve package versions,
checksums, owner edges and resolved features after the authorized revision
mapping. Metadata moves from 1,566 to 1,568 packages because published Knot
`eabd4434` retains separate Fleece 0.5.0 and LayoutDom 0.1.1 identities at
Genet `965b64e2`. The five selected Cambium roots retain the same 605-package
dependency closure, which activates neither JavaScript engine. The committed
lock SHA256 is
`69eb637d64e7387179cda95b1a2e80768e076d95da29c24b9203d0c573551cd5`.

On Fedora ThinkPad, Cargo 1.98.1 runs with one build job and one test thread.
The four-package gate passes Cambium 257, Rootstock 72, native accessibility
23 and browser mirror 10, with zero failures and two existing ignored doctests.
All six native-host integration targets pass 63 tests; the Wasm `a11y_page`
example compiles with its existing getrandom setting. These are automated
tests and compilation, without a new physical window, browser operation or
human assistive-technology receipt. Source, manifest and lock are stable across
each run. The first package compile is interrupted when a separate Cargo owner
appears; it runs no tests and remains unqualified. The unchanged serial retry
supplies the complete result.

Normal Cargo Git checkouts contain the exact clean published Genet and Knot
commits. The audit finds no local source replacements or Git URL rewrites.
Raw receipts use prefixes `thinkpad-forms-20261008-mere-packages-retry1`,
`thinkpad-forms-20261008-mere-native-host` and
`thinkpad-forms-20261008-mere-web-wasm` under
`Code/testing/genet/forms/thinkpad`. Graph and provenance qualification are
`thinkpad-forms-graph-qualification.json` and
`thinkpad-forms-20261008-provenance.json` under `Code/testing/genet/forms`.
The later merge of published Mere `9ed44a5e` changes only documentation and
packages outside the tested closure; its manifests, lock and tested closure
remain identical. Mark's publication authorization includes the normal main
push carrying the preserved shared history.

The remote Mere worktree is required by an existing primary-checkout Knot
lane; Turnstone's worktree protects primary untracked receipts. Both use
stable repository targets. Worktrees can be removed after publication and
evidence transfer when they have no live owner. The primary work and other
agents' targets remain preserved.

### 2026-09-29 generated accessible-name adoption

**Status:** implemented and verified against the published source. All 207 tests
in the Rootstock, native host, native accessibility and Mesquite gate pass; the
standalone Graphshell Wasm check passes. Locked offline metadata audits verify
26 root and 19 web Genet package identities, with no local source substitution.

The current-family root and standalone Graphshell web manifests now pin Genet
`c5470fcbc12805f0369c70f34a18178158fbe2d5`. Rootstock's retained style owner supplies
generated before/after text to both the document accessibility projection and
the native AccessKit adapter. Mesquite therefore sees the same generated names
as native accessibility, including attribute-driven changes after relayout.
Genet owns generated text and named decimal counter evaluation; Mere consumes
the resulting text rather than reimplementing CSS or accessible-name rules.

The root lock is the exact current-family revision substitution over the incoming
working lock, preserving unrelated Gaz changes. The standalone lock additionally
adopts already-landed text-boundary and diagnostics dependencies: it adds
`genet-text`, swaps Cambium's direct unicode-segmentation edge, and adds workspace
`mere-apparatus` plus Mesquite's dependency on it. There is no other
package/version/source drift. Package/source classification
is recorded under `Code/testing/cambium/generated-names`. Older Knot-owned Genet
identities and the separately qualified legacy `genet_web_smoke` manifest remain
unchanged. A native-adapter/Mesquite fixture checks generated names and attribute
updates; this is automated coverage, not human screen-reader acceptance. The
[testing receipt](../testing/local-genet-development.md#2026-09-29-generated-name-adoption)
records exact commands and the archived ignored standalone lock.

### 2026-09-29 semantic selector adoption

That checkpoint pinned Genet `19c206873ab08ae227217892d9e74d0df18b349a` coherently in
the root and standalone web manifests. Relative to the verified scroll-repair
revision `7a60ad79`, only Taproot code and documentation change: role/name
selectors can consume Genet's existing computed accessibility projection.
Rootstock exposes that projection; Mesquite uses it and rechecks a held target
after scrolling. Class/text matching and product coordinate transforms remain
compatible. Custom-leaf semantics and human AT acceptance remain separate.

The final integration preserves primary `32edc2ad`, including Insigne, Knot
`855cb75d` and the formatting-line repair. All 199 Rootstock/winit-host/Mesquite
tests and standalone Graphshell Wasm checking pass. Both locks preserve every
incoming package/version/source tuple after mapping the Genet revision; only
Taproot's dependency on document-session-api is added. Existing Knot-owned
legacy Genet identities remain separately qualified. The old-matcher negative
control fails the referenced-name test; restored source passes. Native smoke
evidence and current limitations are recorded in the
[Cambium architecture](2026-09-03_cambium_architecture.md#dom-selector-parity-2026-09-29-implemented-and-verified)
and the [diagnostics plan](../../mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md).
Raw receipts live under `testing/cambium/semantic-observation` in the shared Code
root. The dated compatibility checkpoints below retain their original scope.

### Historical September 13 boundary

Cambium is now a Mere workspace family under `crates/cambium/`; it is not a
Genet workspace subtree. Mere owns the current `cambium`, `meristem`,
`sprigging`, and `workbench` paths, while pinning Genet seam packages at one
immutable `genet.git` revision (`7baa554c66f966295ee945d908738b635afe136a`).
Errand is likewise Mere-owned at workspace version 0.3.4.

That pin moved from `101d9e9ade8671564e723443d9f0498e899a33f1` on 2026-09-14,
eight commits forward, to take `genet-probe`'s new name `taproot`. The
compatibility table below was not re-validated at the new revision; the
2026-09-13 receipt in the architecture doc still names the old one, which is
what it measured.

The external viewport bridge uses that revision's content-box geometry and typed
used-color queries. Genet's vendored Parley and Taffy patch entries carry the same
revision; upstream Parley 0.10 has the same version but lacks the engine's local
extensions. Netrender and the paint-list family align to
`3961aca919f707ab09a786379eb4ce8bb121258e`, matching Genet's own dependencies.
Mixing that revision with Mere's former Netrender pin produced distinct Rust
types when ignored sibling patches were absent. Root and standalone host manifests
must keep these families aligned.

| Boundary | Current source in this workspace | State |
| --- | --- | --- |
| Cambium family | `crates/cambium/` workspace members | Mere-owned |
| Genet DOM seams | `genet-scripted-dom`, `layout-dom-api`, and `genet-static-dom` pinned to the revision above | Genet dependency boundary |
| Errand | `crates/system/errand` 0.3.4 | Mere-owned protocol AST dependency |

Downstream consumers' pins and registry publication state need verification in
their own repositories or registries; this document records only the current
Mere workspace boundary.

## Custom-leaf protocol

Cambium emits Genet's neutral `<custom-leaf>` element and related attribute
vocabulary. Genet temporarily accepts `<chisel-leaf>` as a read-side
compatibility alias for older documents.

## Direction rule

Cambium may depend on Genet seam crates. Genet engine crates must remain free
of Cambium, Meristem, and Sprigging dependencies. Reference applications such as
Pelt may depend on all three.

## 2026-09-27 accepted text bounds publication

The owning Genet fix under Isocosm rulings 329/379 is accepted at exact commit
`7b48f94d7a742840b527205d82a37da958240d73`. It separates font-content bounds
from formatting line boxes and anchors wrapped inline decorations to their
own font/baseline. Main's retained motion and restyle preparation are included;
the combined owner receipt passes 876 affected tests (6 existing ignored),
200 boundary tests and a current-family local Isometry detector pair of
187/0 short rows against the published baseline's 187/39. That local pair was
not a committed portable consumer repin. Source and evidence qualifications
live in `genet/design_docs/2026-09-25_line_box_model_plan.md`.

Mere is preparing publication of that exact Genet commit in its 27 root and
7 standalone Graphshell web manifest rows. NetRender remains `9607d16`,
netrender-vello 0.10.1, wgpu 30 and the accepted pre.2 compute patches remain
unchanged. Direct Cargo uses published sources; inactive machine-local configs
are not supplied. Portable graph/lock classification and the bounded host,
Cambium, Sprigging and standalone-web checks are pending. The separately
pinned old Fleece/layout-dom-api lineage through Knot is pre-existing and
outside this pin's acceptance claim. Downstream Isometry publication follows
only after Mere's reviewed commit is pushed.

**Preparation note, 2026-09-28:** a long first metadata invocation was initially
misidentified as dependency-solving cost. The trace had reached patch
registration while the new Git checkout still lacked Cargo's `.cargo-ok`
marker. Its files carried the current attempt's timestamps; the prior cached
revision's manifest-to-marker interval was eight minutes and forty-five seconds.
That supports unfinished checkout preparation, not a dependency conflict.
Interrupted attempts remain recorded as interrupted preparation. Compare this
phase/progress before diagnosing a resolver failure; acceptance still requires
completed locked metadata and the consumer checks. The external candidate lock
is an unvalidated, source-backed input until Cargo accepts it unchanged.

**Publication gates, 2026-09-28:** Cargo 1.98.1 completed `metadata --offline
--locked` and accepted the external source-backed lock unchanged; that exact
lock is now the primary lock. The graph has 1,524 packages (previously 1,523),
including 24 current Genet packages. After the exact revision substitution,
the only node/feature/dependency-edge changes are the new `genet-text` 0.1.0
package and the source-declared `genet-documents` edge through it. The two
old `34626a6` Fleece/layout-dom-api packages remain. Both copied-metadata
controls reject an outside path and an old current-family Genet source.

The all-target checks for `cambium-genet-winit-host`, `cambium` and `sprigging`
pass using the primary published lock. Standalone Graphshell web's Wasm check
also passes with its separately validated ignored lock and only the required
`getrandom_backend="wasm_js"` target cfg. That lock has exactly the revision
substitution from the preserved prior portable lock; current unfiltered
metadata has 527 nodes and 18 Genet packages, with no documents/text package.
The historical provenance's 432-package count has no recorded filter/command
and is not reused as a current count. Native wgpu remains 30.0.1; the independent
web lock retains 30.0.0. Both retain NetRender `9607d16` and netrender-vello
0.10.1. Web source-detector controls also reject both deliberate faults.

Raw commands, compiler identity, original and accepted locks, source audits,
controls and logs are in `testing/mere/receipts/2026-09-27/genet-text-publication`
under the shared Code root. An initial direct-Cargo compile invocation selected
rustc 1.97.1 for dependencies and failed on existing 1.98.1 artifacts; the
retained successful commands explicitly select the repository's 1.98.1
toolchain throughout. No target cleanup or source adjustment was needed.
The pre-existing `reader.rs` bytes remain unchanged. These are native/Wasm
compile and dependency-source gates, not new headed, browser or WPT receipts.
Isometry's published-pin text-row gate remains downstream work.

## 2026-09-28 inline scroll bounds adoption (verification in progress)

Primary Mere is preparing exact Genet
`7a60ad7965a1ae81292211b405a53210c554f70c`, whose retained formatting-line
query repairs the scroll-range gap exposed after the text-bounds publication.
The owner design and repair evidence live in
`genet/design_docs/2026-09-25_line_box_model_plan.md`. This update is not yet
committed or accepted for publication. The separate pre.4 lane remains frozen
at held checkpoint `387a8dd2`; this work retains primary's pre.2 compute patches.

Rootstock now consumes `LiveryLayout::inline_scroll_bounds`: the container's
own line bounds, every retained fragment for a descendant, and line bounds
owned by unclipped descendants all contribute in layout coordinates. The
existing end padding/border arithmetic is preserved. A descendant's border
fragments contribute before its clipping boundary; its lines and deeper
descendants do not escape that boundary. The existing non-visible overflow
predicate is shared with `content_clip`, without mixing its painted rectangle
coordinates into this layout-space calculation.

The original hover test still requires a 12-pixel offset, now also asserted
before hover alongside a nonzero initial range. New tests cover own and
descendant line bounds, hidden/clip/auto/scroll descendants, and later font
fragments extending beyond short line boxes. The full Rootstock suite passes
44 tests. Four deliberate source controls separately omit container lines,
omit descendant lines, retain only the first fragment, and omit the clipping
prune; each compiles and fails its expected assertion. Exact source restoration
is followed by another 44-test pass. These are correctness results, not timing
or exclusive-GPU measurements.

The 27 root and seven standalone web manifest rows move together. Both locked
graphs accept exact source-substituted candidates without resolver changes:
native metadata remains 1,524 packages with 24 current Genet packages and two
old Knot-owned identities; web remains 527 packages with 18 Genet packages.
Node features and dependency edges are unchanged after mapping the revision.
Both detectors reject an outside path and an old current-family source.
Root/web locks retain 1,651/527 packages, WGPU 30.0.1/30.0.0, NetRender
`9607d16f` and netrender-vello 0.10.1. All 385 recorded pre.2 patch files retain
their bytes. The initial offline cache miss and completed pinned Git checkout
preparation are preserved as separate attempts.

The new optional accessibility description field requires two Reader
initializers and the web-host blank test initializer to specify `None`.
Reader's former dirty state had no normalized content delta: its archived raw
preimage contains 956 CRLF lines and four LF-only lines. The adoption preserves
those bytes and inserts exactly two LF-terminated lines; it does not normalize
the file or absorb unrelated edits.

Full affected-host tests, workspace verification and the native/web consumer
compile gates remain pending. After the focused controls, C: had about
1.54 GiB free; existing remaining host debug artifacts alone were about
1 GiB before new libraries and link temporaries. Further builds are held for
adequate headroom, without reducing proof scope or deleting evidence.
Commands explicitly use Rust 1.98.1, four jobs and the stable Mere target;
incremental output is disabled for this bounded run. Receipts are under
`testing/mere/receipts/2026-09-28/inline-scroll-adoption` in the shared Code
root. Main publication, pre.4 reconciliation and S13 remain separate gates.

## 2026-09-29 resumed verification before main integration

Disk headroom is restored. The five checks held in the September 28 entry now
pass at the same runtime source and locked Genet `7a60ad79` graph: affected
host/Mesquite tests, Reader all-target check, portable workspace verification,
native Cambium/Sprigging/host all-target check, and standalone Graphshell web
Wasm check. The host/Mesquite run has 156 passing tests, zero failures and three
existing ignored tests across 25 result blocks. Each accepted command records
1,996 unchanged source/lock hashes, matching the earlier restored Rootstock
proof. The original 44-test passes and four deliberate assertion failures
remain the focused repair evidence; they were not needlessly repeated.

The first resumed host attempt stopped before compilation because the offline
registry cache lacked `petgraph`. A separate `cargo fetch --locked` restored
the exact cache without changing locks or sources, and the preserved offline
retry passed. Commands continue to use Rust 1.98.1, four jobs, the stable Mere
target and disabled incremental output. The web check additionally uses only
`--cfg getrandom_backend="wasm_js"`.

The first full workspace check exited successfully, but another task briefly
edited and restored four shared source files while it ran. That attempt is
retained as qualified evidence, not the accepted gate. The other owner reports
the interval as 2026-09-29 04:09:16.765 through 04:10:39.248 UTC; root separately
verified the restored content. After the first check ended, all four files
matched HEAD and the accepted runtime hashes. Advancing only their mtimes
forced Cargo to recheck Rootstock, Mesquite and the winit host. The separate
`workspace-verify-restored-selector` run passes. Host tests and the Reader check preceded
the interval; native and web checks followed restoration.

This is a verification checkpoint before integration with newer
`origin/main`, not permission to publish an older main. The remote now carries
separately approved canvas, physics and web work, including additional web
packages, features and a newer Wasm binding version. The recorded 1,524/527-node
graph comparison belongs to this bounded source snapshot. Integration must
classify those upstream changes alongside the exact Genet revision substitution
and rerun the affected tests and native/web graph checks on the combined tree.
Pre.4 reconciliation, S13, headed/browser receipts and downstream Isometry
publication remain separate gates. `resumed-checkpoint.json` in the same receipt
directory seals this outcome without altering `blocked-checkpoint.json`.

## 2026-09-29 combined checkpoint with a31b9a14

The pending merge combines local scroll adoption `8eca3e4c` with incoming main
`a31b9a14`, preserving upstream runtime work and the tested Genet `7a60ad79` pin.
Native metadata remains 1,524 packages with 24 current Genet packages and the
two existing Knot-owned identities; its only added edge is web-host to Mesquite
under `cfg(target_arch = "wasm32")`. The root lock retains 1,651 identities and
all 385 recorded pre.2 patch files retain their bytes.

Web metadata/lock grows from 527 to 636 packages: seven required Wasm-family
updates and 109 additions reachable from the three approved local host roots.
Nineteen existing nodes gain features/edges without losing prior ones, including
the requested `Location` and `UrlSearchParams` features. The filtered Wasm graph
has 445 reachable nodes; these are graph counts, not payload measurements.
An incidental offline `libredox` downgrade was rejected and prior 0.1.25 restored.
The accepted ignored web lock and all attempts are preserved. Native/web WGPU
stay 30.0.1/30.0.0, NetRender stays `9607d16f`, and netrender-vello stays 0.10.1.
Both fresh source detectors reject an outside path and an old Genet revision.

Tests pass: 202 Rootstock/host/Mesquite tests with four ignored doctests;
261 Pictograph unit plus two headless Vello tests; 88 Seiche default-library
and eight no-default runtime tests. Both retained-layout cases and the GPU
lost-crossing-edge control pass. Workspace verification, Reader all-target,
native and standalone Wasm checks pass. These are correctness results on a
shared device; pre.4 tensor/release and S13 acceptance remain separate.
All commands preserve 3,034 tracked-file/ignored-lock hashes. The earlier host
wrapper has hashes only; seven later runtime/check runs also preserve mtimes.

Evidence: `testing/mere/receipts/2026-09-29/scroll-main-integration/combined-checkpoint.json`
under the shared Code root. This accepts only the a31b9a14 checkpoint, pending
independent review and local commit. Remote main advanced to `ee77bf23` during
verification; its Personae/Insigne migration must be integrated and the affected
dependency checks refreshed before publication or downstream adoption.

## 2026-09-29 ee77bf23 follow-on verification

The verified merge of `ee77bf23` into `14f5f9c9` retains tested Genet `7a60ad79`
and upstream's Insigne migration. Default native metadata keeps 1,524 packages:
four Knot identities move to `855cb75d` and eight nodes gain Insigne edges.
Web keeps 636 packages (445 in the filtered Wasm graph), adding only the
Pandect-to-Insigne edge. Registry identities/checksums and all 385 pre.2 patch
files remain unchanged. Existing legacy Genet and Knot-site identities remain
explicitly classified. Five default-graph fault controls reject the wrong source
or an outside local path.

Exact package, feature, dependency and local-file comparisons preserve the five
Rootstock/host/Mesquite/Seiche runtime cones, carrying their a31b9a14 receipts.
Pictograph reaches changed Personae files, so its 263 tests were rerun and pass,
including retained-layout cases and the headless crossing-edge control. The
Insigne/Personae/Notochord all-feature suite passes 233 tests. Native workspace
and standalone Wasm all-target/all-feature checks pass; only the existing
`mere-linked-data` native failure is excluded. All-feature metadata selects
1,651 native and 636 web packages within the unchanged locks; four additional
source controls reject both faults. Metadata includes `mere-linked-data` and
does not establish compilation of that excluded package.

Every follow-on gate retains 3,034 source/lock hashes and unchanged mtimes.
Shared-device rendering establishes correctness, not performance. Evidence is
`testing/mere/receipts/2026-09-29/scroll-insigne-integration/final-checkpoint.json`
under the shared Code root. Pre.4, S13 and downstream adoption retain their
separate gates.
