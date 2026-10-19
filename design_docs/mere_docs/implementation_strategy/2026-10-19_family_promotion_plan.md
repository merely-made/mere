# Foundation family promotion

**Date:** 2026-10-19
**Status:** Completed source promotion and consumer cutover. Authorized by the owner after the dependency and ownership review. Source extraction, independent validation and consumer cutover are separate gates.
**Receipt budget:** 100 MB total across this migration. Keep compact command summaries and source hashes; do not copy model weights, build trees or historical audit receipts.

## Decision

Mere remains the graph application platform. Graphshell, Djinn, Cambium, Conatus,
the graph kernel, canvas and application-specific integration stay in Mere.
Promote the reusable foundations without changing their public APIs, crate names,
licenses, stored formats, authority rules or default feature semantics.

The Eidetic repository owns Muniment, Chartulary, Eidetic, Hagiograph,
Stickleback, Luggage, `mere-proofs` and Servitor as separate crates. Storage-only
consumers do not acquire replication, an updater or participant execution.
Stickleback retains its policy-before-insert contract; domain consumers supply
operation meaning and authorization. Luggage retains a portable release identity
core and a separately selected native updater. Servitor evaluates host-supplied
authority, gates attributed graph petitions and reduces run observations; hosts
still execute work and own lifecycle persistence and effect reconciliation.

Armillary gets a separate repository. Its actor boundary, message correlation,
worker pool and wake contract remain host neutral. ESP gets a separate repository
for model execution, with optional Armillary and Eidetic integrations. Gemot gets
a separate repository for the governance/community family, including Mien,
Mooting, Moothold and Commons. The Mere Moot port remains an application adapter.

Errand becomes the continuing home of the Smolweb repository and its thirteen
protocol crates. Import Errand's implementation history into that existing
repository, preserving protocol crate identities and licenses. This move does
not invent a universal transport interface or transfer protocol semantics to
Murm. Reusable carriage factoring into Murm remains consumer-driven development.

## Phases and done-conditions

### 1. Establish sources and extract

Update clean main checkouts before extraction. Retain the source revision and
hashes for every moved file. Preserve filtered Git history in new repositories;
preserve the continuing Smolweb history when importing Errand. Existing dirty
work is outside the migration unless explicitly identified as part of it.

Done when extracted crate source matches the donor, with documented manifest,
documentation and integration-placement exceptions, and each new workspace has
explicit package metadata, dependencies, licenses, toolchain and CI.

### 2. Remove integration back edges

ESP's lexical/BM25 fusion test belongs with Mere's `eidetic-search` integration.
Gemot's capsule-library example composes Mere's native WIT host and belongs with
Mere's Moot port. Preserve both checks rather than making promoted libraries
depend back on Mere merely to exercise the composition. Protocol conformance and
standalone library tests remain with their owning families.

Done when promoted production libraries have no dependency on Mere's aggregate
library or ports, and integration targets have a maintained receiving location.

### 3. Validate and publish suppliers

Run independent focused tests for Armillary, Eidetic, ESP, Gemot and Errand.
Exercise portable core feature configurations where supported. Preserve real
network, large model and headed acceptance as separately scoped checks; this
source-home change does not establish new behavior or erase existing failures.
Publish suppliers and consume one immutable revision per family. Retain the
existing shared dependency source identities and owned p2panda patches.

Done when named independent gates pass and canonical repository URLs and exact
supplier revisions are recorded. No registry release is implied.

### 4. Cut over consumers and retire copies

Update Mere's workspace membership, direct path dependencies and root patches.
Redirect immutable older suppliers that still name moved packages through
`mere.git` to the new canonical family. Update Murm's foundation dependencies,
then other affected sibling manifests without disturbing unrelated work.
Refresh lockfiles and check representative consuming workspaces. Remove the
donor implementation directories only after the receiving source and history
are secured. Do not leave two maintained copies or local-path-only receipts.

Done when no live workspace member or path dependency points to a removed
implementation, selected package graphs resolve a single canonical source,
consumer checks pass, source and documentation changes are committed and pushed,
and all remaining gates and retained build locations are stated.

## Findings

- 2026-10-19: Mere was clean at `24c669d2` and fast-forwarded to `23541321`
  before extraction. Smolweb was clean and synchronized with its remote.
- 2026-10-19: Murm consumes Muniment, Stickleback, `mere-proofs` and Luggage
  through Mere. All four belong in the new foundation cutover.
- 2026-10-19: ESP's only `eidetic-search` dependency is a cross-library test.
  Gemot's native capsule example depends on Mere's app-host and WIT assets.
  These are integration targets rather than production-library dependencies.

## Progress

Supplier extraction and publication completed. All 301 moved Rust source files
match the donor after line-ending normalization and the explicitly recorded
integration-target receiving-package/WIT changes. Source inventory and compact
receipts live in `support/family-promotion/`. The 395 donor files matched their
recorded hashes before retirement. Donor implementation directories have been
removed. All 21 consuming workspace roots resolve and revalidate their lockfiles
with one canonical source for every promoted package they consume.

| Family | Published revision |
|---|---|
| armillary | `6827559937f07f78a6a830dfbfcaa54210dd093c` |
| eidetic | `de6b1dce75dd6725d72c79b829e4878c7903a381` |
| errand | `dce1f08546b8475e42a5a01ca1ea18794c6e9b97` |
| esp | `a2561b0b9766a4ad4d9924c02194bb414667b9ef` |
| gemot | `6a5023a9ca478ebcc342dd9b7c9e418ccacfc6b5` |

Independent gates passed: Armillary unit/doc tests; the full Eidetic workspace;
Luggage's Wasm core; Errand default and all-feature protocol workspaces; ESP's
portable actor/persistence and CPU decoder/model-session configurations; the
Gemot workspace against canonical Git suppliers. External model, GPU, headed
and physical acceptance are unchanged and outside this source-home receipt.

The first ESP test run exposed two decoder tests lacking feature requirements;
the promoted manifest now declares them. A smaller Gemot build exposed an
Errand protocol feature inherited from Mere that the initial extraction omitted;
the corrected manifest retains Finger's explicit client feature. Failed receipts
remain beside the passing follow-ups.

The normal-library graph is the layering contract. Eidetic's existing carrier
integration tests may consume Murm, and Murm's separate Signalman application
workspace may consume Mere UI. Those test/application edges do not make the
Eidetic or Murm production libraries depend on Mere application composition.

## Consumer qualification and limits

The source graph audit covers Mere, Graphshell web, three Distillery roots,
Cleromancy and its core, Knot, mer3ly and repo-graph, Murm native and Signalman,
Turnstone, Woodshed and three application/proof roots, and four Isocosm roots.
The exact roots and immutable package sources are in
`support/family-promotion/consumer-source-validation.json`.

Executed tests include 539 Eidetic, 10 Armillary, 334 default-feature and 416
all-feature Errand, 105 portable and 198 decoder ESP, 242 Gemot, 762 Mere
kernel/Seiche/Graphshell-client, 17 Cleromancy core and two relocated search
tests. These configurations overlap; their counts are not a unique-test total.
Ignored tests remain recorded in `verification-summary.json`.

Mere's platform library, Wasm Graphshell, Luggage's browser core, Woodshed's
core/graph, Knot document and Isocosm domain compile. The relocated capsule
example and its WASI guest also compile; this does not claim native/WIT/browser
end-to-end execution. The lexical integration test stays with `eidetic-search`;
the capsule application composition stays with Mere's Moot port.

The unfiltered Murm run hung on
`a_killed_peer_off_the_overlay_reads_not_connected_when_its_connection_closes`.
The follow-up executed 169 tests with that single shutdown case explicitly
filtered and one child fixture ignored. Both receipts are retained. This gate
remains open; the source move does not claim to repair transport shutdown.

Mere's whole-tree license-header audit still reports 658 missing Exhibit A
headers. Comparing the original donor revision with the final tracked sources
finds the same 658 and zero newly missing headers. Existing licensing grants and
third-party notices travelled with Eidetic and Errand. No registry release or
license-policy change is part of this migration.

Active Eidetic, Armillary and Gemot documents followed their source owners.
Mere retains its application plans and historical checkpoints with owner links.
Transport factoring into Murm remains a separate follow-up. Broad ownership
migrations inside the retained graph runtime are not declared complete here.

All Cargo checks reused `C:\t\cargo-targets\mere`; no isolated Cargo home was
created. An Isocosm worktree was necessary to publish the adoption without its
unrelated unpushed source commit. The adoption was integrated back into the
primary checkout, its original product paths were verified unchanged, and the
worktree was removed. The unrelated product commit remains unpublished. Retained compact receipts, including ignored
local preservation backups, remain below the 100 MB budget.

## Publication

All five supplier repositories are published. Their tested source cohort is
listed above; later documentation-only commits do not change those pins. Seven
consumer adoptions are pushed, with exact commits in
`support/family-promotion/consumer-publication.json`. This Mere commit retires the
donor implementations and records the canonical source graph and integration moves.
The local Errand checkout remains named `repos/smolweb`; the GitHub repository is
now `merely-made/errand`, retaining the existing repository identity and history.

The document-judgment check covers this new plan and recognizes moved documents.
It still fails on the original 19 missing records and an unchanged snapshot
whose checkout digest differs from the checker constant. The baseline comparison
records zero newly missing documents; the historic snapshot was not edited.
