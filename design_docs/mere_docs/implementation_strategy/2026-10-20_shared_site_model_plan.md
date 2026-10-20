# Shared site model extraction

**Status (2026-10-20):** in progress. First slice authorized by Mark: extract the
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
   Retained receipt cap: 100 MB across repositories. Reuse the approved stable
   Mere Cargo target; no isolated worktree or Cargo home is planned.

## Findings (2026-10-20)

- Knot's `crates/knot-site/src/lib.rs` combines portable records and validation
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
