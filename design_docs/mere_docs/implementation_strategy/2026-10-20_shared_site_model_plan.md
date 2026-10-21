# Shared site model extraction

**Status (2026-10-21):** extraction and optional saved-site revision storage
landed. The optional Moot publication binding is landed, with Gemini,
Scroll and Micron as the first profiles. Snapshot V1 stays unchanged.

## Scope and ownership

`mere-site`, at `crates/domain/site`, owns the portable native-site profiles,
page metadata, manifest validation, and `PublishedSnapshotV1` encoding and digest.
The default build has no filesystem, listener, runtime, editor, identity or
storage dependency. Optional `history` uses Eidetic's Muniment backend seam.
Optional `publication` composes Eidetic typed retention and Gemot's existing
signed contributions and collection authority.
Knot keeps file access, drafts, templates, checked saves and protocol adapters.
Djinn keeps custody, certificates, listener policy and resident lifecycle.

Cross-Moot publication lineage, network delivery and new Errand/Armillary
adapters are subsequent work. Existing V1 limits and profile semantics remain
unchanged; the adapters reuse existing publication and storage authority.

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

## Saved-site revision slice (2026-10-21)

- Keep checkpoint metadata and the caller-assigned local history identity in a
  separate V1 journal envelope. Immutable blobs retain exact snapshot V1 bytes.
- Use `Journal<SiteRevision>` and `BlobStore` over a caller-supplied Muniment
  backend. Journal commit checks both the previous journal and the blob inside
  `Backend::transact`; unsupported transactions and competing writes refuse.
  Failed journal persistence may leave an unreferenced blob. Collection policy
  is deferred; no recorded revision is pruned implicitly.
- Hosts configure journal byte and revision capacity. Hitting capacity refuses
  the new checkpoint while retaining prior entries. Repeated explicit
  checkpoints record separate occurrences while sharing identical blob bytes.
- Knot supplies an optional history store and explicit saved-site checkpoint
  command. Its folder binding identifies local history, not a public site or
  persona. Checkpointing reads saved files, including hand edits, and does not
  start a listener or write working files. A history browser/restore workflow,
  draft keystroke log, forks, federation and serving actors are later slices.
- Done when two distinct checkpoints survive backend close/reopen with exact
  bytes and digests; failed blob/journal writes expose no new revision; corrupt
  blobs refuse; competing writers cannot silently overwrite each other; Knot's
  explicit command refuses unsaved work and works without local serving.
- Receipt cap remains 100 MB across repositories. Reuse the stable repository
  targets. Mere's concurrent root manifest/lock edits require the temporary
  approved `Code/worktrees/mere-site-revisions` worktree, removed after integration.

### Findings (2026-10-21)

`muniment` at the existing Eidetic pin already supplies transactional directory
and memory backends, immutable blobs and serializable journals. No Eidetic core
change is required. Knot's `Site::publication` reopens saved manifest/source;
that is the checkpoint capture boundary. Desktop source ownership and server
policy remain in Knot and Djinn respectively.

## Moot publication binding (2026-10-21)

**Status:** landed and qualified on integrated main. Gemini, Scroll and
Micron are the first profiles.
Optional `mere-site/publication` composes existing Gemot and Eidetic owners;
the default model and local checkpoint feature stay independent.

- Bind exact V1 bytes to Eidetic's existing schema and typed-content APIs for
  explicit host retention. Verify retained bytes before authoring through Gemot's authorized
  stable-identity contribution command. Collection selection remains an explicit
  existing Gemot command, with its observed heads and capability checks.
- Open an explicitly named contribution at an exact authorized collection
  version. Refuse foreign, ineffective, stale, private, corrupt or unsupported
  content. Recheck current selection after local content resolution. Return
  native pages and metadata, including the profile's root index mapping.
- Done when all three profiles retain exact fixture bytes and metadata after
  backend reopen with the author absent; signed collection selection and forks
  use existing Gemot history; unauthorized contribution, withdrawal and stale
  selection refuse; the four original V1 fixtures still pass.
- This is a local owner-composition gate. Network serving, peer acquisition,
  hosting promises, signed cross-Moot publication lineage and actor lifecycle
  remain separate gates. No timestamp-based latest-revision policy is added.
- Retained receipt cap: 100 MB. Concurrent root manifest/lock work requires
  `Code/worktrees/mere-site-moot-publication`; reuse the stable Mere target.

### Publication findings (2026-10-21)

- `gemot::moot` already supplies signed stable-identity contributions,
  capability-gated causal collections, forks and withdrawal. Site publication
  can cite those facts without a second community identity or ancestry log.
  A collection may contain several sites; the host explicitly selects a share
  at an observed version instead of inferring one from timestamps or titles.
- `eidetic::typed::TypedPayload` supports a custom byte codec, so snapshots
  retain their original canonical JSON and BLAKE3 identities. The schema includes
  all four existing model profiles; only the publication adapter is limited to
  the three selected initial profiles. Native metadata is never flattened.
- Eidetic's `manifest:` and `blob:` keys do not fit the directory backend's
  portable filename restrictions on Windows. Disk qualification uses redb;
  local history's separate filesystem-safe keys remain unchanged.
- Current selection is checked around content resolution, not held as an atomic
  serving grant across owners. An opened site is a point-in-time read. Network
  hosts must arrange their own revalidation/revocation behavior and hosting policy.

## Extraction findings (2026-10-20)

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

- 2026-10-21: optional history passes six persistence/failure/concurrency cases
  and all three original V1 compatibility cases; the default feature-disabled
  build also passes its three fixtures.
  [Supplier receipt](../../../support/receipts/shared-site-revisions/supplier.json).
  Knot consumer qualification remains in progress.

- 2026-10-21: revision storage and the Knot consumer are landed. Knot source
  `5297224` and receiving record `e24ee9e` are published; the combined Hest/Cambium
  receiving cohort retains every upstream Git source outside the exact portable
  model contract. Six shared history controls, three original V1 fixtures,
  18 standalone site cases, the desktop checkpoint case and all nine launch
  cases pass. The normal default-feature desktop check also passes.
  [Consumer qualification](../../../support/receipts/shared-site-revisions/consumer.json).
  The temporary supplier worktree was removed; ordinary stable Cargo targets
  were reused and no new isolated Cargo home was created. A history browser,
  restore UI, replication and serving adapters remain outside this slice.

- 2026-10-21: optional publication binding passes five focused cases (including
  three native profiles), all three original V1 compatibility cases, six local
  history cases and the compiling Eidetic retention example. Repeated with
  `serde_json/preserve_order` to qualify fixed schema identity; the feature-disabled
  build also passes its three compatibility cases. Disk content reopens in redb
  after author handles are dropped; signed community facts are admitted to a
  separate in-process replica. This does not claim a new OS-process, network or
  two-machine serving receipt. No supplier repins are required.
- 2026-10-21: source `943f3f564` is integrated and pushed on Mere main, preserving
  the concurrent root dependency and diagnostics work. The locked receiving
  gate passes 14 tests and the retention doctest with map ordering enabled;
  strict no-deps Clippy also passes on the supplier. Gemot's owner plan records
  the boundary at `6485b8c`. [Qualification](../../../support/receipts/shared-site-publication/qualification.json)
  and [integration](../../../support/receipts/shared-site-publication/integration.json)
  retain exact scopes. The temporary worktree and branch are removed. Cargo-home
  cache junctions are removed; automatic approval review rejected deletion of
  its 110,906 metadata bytes as "blocked by policy." No isolated target remains.
  Live protocol serving and peer acquisition, host revalidation/revocation,
  actor wiring and cross-Moot signed publication lineage stay open.
