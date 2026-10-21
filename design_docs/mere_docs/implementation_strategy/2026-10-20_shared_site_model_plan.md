# Shared site model extraction

**Status (2026-10-20):** landed. First slice authorized by Mark: extract the
shared model from Knot, adopt it in Knot and Djinn, preserve snapshot V1.

## Scope and ownership

`mere-site`, at `crates/domain/site`, owns the portable native-site profiles,
page metadata, manifest validation, and `PublishedSnapshotV1` encoding and digest.
It has no filesystem, listener, runtime, editor, identity or storage dependency.
Knot keeps file access, drafts, templates, checked saves and protocol adapters.
Djinn keeps custody, certificates, listener policy and resident lifecycle.

Site identity, ancestry, revision logs, replication and new Errand/Armillary
adapters are subsequent work. Existing V1 limits and profile semantics remain
unchanged. This slice does not invent a new publication or storage authority.

## Phases and done-conditions

1. Capture pre-extraction canonical bytes and BLAKE3 digests for every V1 profile,
   including Unicode, CRLF source and native metadata. Move the model without
   changing that encoding. Done when the old and shared implementations accept
   the same fixed fixtures and produce identical bytes and digests.
2. Adopt the shared model through Knot's site library and Djinn's resident
   service. Done when consumer model types have one Cargo identity, existing
   site tests pass and focused resident restart/refusal tests pass.
3. Update canonical ownership docs and record compact validation results.
   Retained receipt cap: 100 MB across repositories. Reuse approved stable
   repository targets. Any isolation is temporary and removed after integration.

## Findings (2026-10-20)

- `knot-editor/crates/knot-site/src/lib.rs` combines portable records and validation
  with filesystem authority and protocol handlers. Its `PublishedSnapshotV1`
  canonical JSON is already used for exports and resident handoff.
- `ports/djinn/src/resident_site.rs` already retains Gemini snapshots, preserves
  TLS identity and supports opt-in restart. It currently imports the model from
  Knot; its certificate/custody/lifecycle behavior is outside the extraction.
- Native protocol metadata must survive intact. The existing Errand `Source`
  does not express the complete Scroll metadata contract.

## Progress

- 2026-10-20: reviewed current source, dependency roots and clean checkout state;
  started the pre-extraction V1 compatibility gate.
- 2026-10-20: original Knot code accepted all four fixed canonical fixtures;
  captured its digests before extraction. The shared crate passes three focused
  compatibility tests covering those exact bytes/digests, old manifests without
  a format, and invalid/noncanonical input. Knot's consumer type check uses the
  shared type directly. Consumer and resident qualification are in progress.
- 2026-10-20: Knot's default site suite passes 17 tests against the published
  supplier. Its optional Retinue suite passes 22 tests, including actual local
  Micron serving/replacement/stop. The unchanged donor reproduced a Windows
  extended-path-prefix assertion in the export test; canonicalizing both sides
  fixes that test without changing export behavior. Rebased the consumer over
  upstream cohort updates and published it as Knot `0fcc522`.
- 2026-10-20: isolated resident qualification in the approved
  `Code/worktrees/mere-shared-site` checkout because the shared Mere checkout
  acquired concurrent renderer edits and an unrelated lockfile change. Reusing
  stable Mere and Knot Cargo targets; no isolated Cargo home.
- 2026-10-20: all eight isolated Djinn resident-site tests pass, including
  exact saved-snapshot/certificate reopening, opt-in restart, malformed handoff,
  failed replacement and connection bounds. Knot's ordinary root lock now names
  supplier `c25c2d441`; its library check and all three V1 fixture cases pass under
  that root dependency set, with original Murm/Netrender revisions retained.
  Compact evidence: [qualification receipt](../../../support/receipts/shared-site-model/qualification.json).
- 2026-10-20: the concurrent source lane integrated upstream into primary Mere
  `67964c3ea`. Final receiving qualification will use that integrated head;
  unrelated documentation and receipt work remains outside this lane.
- 2026-10-20: final integrated-main resident-site gate passes all eight tests
  (221 unrelated cases filtered), with exact preflight lock/source hashes intact.
  The ordinary Knot root cohort was rebased over the separately qualified
  borrowed-lowering cohort rather than rolling it back; its model fixtures also
  pass there. [Integrated receipt](../../../support/receipts/shared-site-model/integration.json).
  Revision logs and new persistence/Errand/Armillary adapters remain subsequent
  slices. Temporary isolation is removed after the scoped source commit.
