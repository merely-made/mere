# Site publication: mere-site, the Djinn publication host and wider smolweb coverage

**Status:** 2026-10-28: cut from `2026-10-20_shared_site_model_plan.md` and the section 13 part of
`2026-08-22_djinn_family_resident_services_plan.md` (ordinary site serving and governed replication), archived in
`archive_docs/2026-10-28/`. Landed: the portable model in `mere-site` (`crates/domain/site`) with snapshot V1 unchanged,
optional saved-site revision storage, the optional Moot publication binding (Gemini, Scroll and Micron first profiles),
the Gemini, Scroll and Micron resident runtimes qualified through native processes, shared browser receiving, client
limits for routed Errand and Micron consumers, the explicit-Submit rule, and the ordinary saved-snapshot Gemini service
in Djinn (local resident receipt 2026-09-13). Open: the persistent-service rollout items, the desktop control and
broker attachment, the acceptance gates beyond native localhost, governed selective hosting with peer acquisition and
cross-Moot lineage, later revision slices, wider protocol coverage and Turnstone's adoption of the effects rule.

**For:** publishing a saved site from an author's machine and keeping it readable after the authoring application exits,
without requiring Gemot membership, paired devices or a moot replica, and then, as a separate opt-in, serving
accepted revisions under current hosting authority. Opened by the 2026-09-11 scope (ordinary saved-snapshot service
first, governed hosting later) and by the shared-model extraction of 2026-10-20 that moved the model out of `knot-site`.
Not here:
- rendering, browser routing, input and certificate conversations and trust projection: `2026-10-28_smolweb_fidelity_plan.md`
  and `turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`, including the browser-owner findings that
  Turnstone's identity-origin code permits Gemini and Titan but not Scroll and that Nematic trust defaults to Unknown
  despite the browser host's TOFU;
- the Djinn update resident (phases A to G), the entry gate, and the open strict-Clippy and fixture-failure items on the
  Djinn port: `2026-10-28_family_update_resident_plan.md`;
- confinement of the publication worker (the tier 3 publication role): `2026-10-28_custody_plan.md`;
- the native Scroll supplier plan (`smolweb/design_docs/implementation_strategy/2026-10-21_native_scroll_server_plan.md`)
  and Errand's client-limits plan (`.../2026-10-22_client_limits_plan.md`): buffered smolweb allocation limits,
  standalone session follow-ups, caller-configurable client bounds and the 4096-byte response-line cap;
- the production publication and hosting records and the author-offline proof, which Gemot's Moot publication plan owns.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **Ownership.** `mere-site` owns the portable native-site profiles, page metadata, manifest validation and
  `PublishedSnapshotV1` encoding and digest; its default build has no filesystem, listener, runtime, editor, identity or
  storage dependency. Optional `history` uses Eidetic's Muniment backend seam; optional `publication` composes Eidetic
  typed retention and Gemot's signed contributions and collection authority. Knot keeps file access, drafts, templates,
  checked saves and protocol adapters. Djinn keeps custody, host identity, certificates, listener policy and resident
  lifecycle. Errand owns wire behaviour and protocol projection; Armillary owns scoped task execution; Retinue owns the
  Micron transport. Source: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:14-27,193-198,425-429`.
- **Snapshot V1 stays unchanged**, with its limits (a page 1 MiB, all raw pages 16 MiB, encoded snapshot 24 MiB) and
  profile semantics. Keep content profiles, transport selection and endpoint policy distinct: `SiteFormat` is not a
  catalog of transports, and the shared model grows only for demonstrated native content requirements with
  compatibility fixtures. Source: same file `:422-429`; `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:625-626`.
- **Ordinary serving is independent of moots.** It must work without Gemot membership, paired devices or a moot replica;
  a moot hosting commitment is a distinct opt-in; advertising availability copies no content and grants no reader
  access or hosting authority. The interface exposes publication, replication and advertisement as separate settings
  with their current state. Djinn must not recreate Gemot's grant evaluation inside a protocol handler; a governed
  descriptor carries references to the domain's verified authority, expiry and revocation. Source: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:562-579`.
- **Host policy stays separate from snapshot V1 and community authority.** Djinn keeps `GeminiListenerPolicyV1` (eight
  admitted connections by default and at most 32; a 2,000 ms default request allowance and at most 30,000 ms; an
  excess accepted socket is shed), the publication-activation gate, and Armillary's `close_and_join`; Knot's admission
  pause and grace are not transplanted. Source: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:37-46`.
- **Require explicit Submit** (Mark, resolved directly): apply the rule before every initial or redirected read
  exchange, so Titan and Spartan submission targets refuse before contact, while the typed submission path and protocol
  wire semantics are preserved. Source: same file `:474-486`.
- **Network hosts own their revalidation.** Current selection is checked around content resolution, not held as an atomic
  serving grant, and an opened site is a point-in-time read; no timestamp chooses a revision. Source: same file `:296-298,316-318`.
- **Serving defaults.** The first implementation binds loopback only; automatic startup is off by default and resuming
  after restart needs explicit `resume_on_restart`; Stop releases the listener and keeps the snapshot, Remove releases
  the lease separately; restore refuses a missing or mismatched identity instead of silently rotating a certificate.
  The broker label `knot-editor` is routing metadata, not authenticated executable identity. Public exposure needs its
  own reviewed deployment receipt. Source: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:604-637`.
- **Before the next adapter slice**, compare the consumer with the protocol crate's existing request, response and
  server APIs; its done-condition names exact native bytes and metadata, request effects, current authority and
  refusal, and resident restart and cancellation behaviour. Native Scroll grammar stays distinct from Gemtext. Source:
  `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:107-114,439-444`.

## Phases

### P1. Settle and verify before persistent rollout
The persistent service contract (a versioned descriptor naming a publication, exact snapshot digest, native paths and
MIME types, byte limits and listener policy; a lifetime handle with replace, stop and status) is built for the ordinary
Gemini route. Seven items were to be settled and verified before rollout is concrete and reviewable; the first slice
touches several of them but none is ticked off.
- Done when: (1) explicit local caller authorization for publish, replace, stop and removal, and a restored descriptor
  cannot silently widen its audience; (2) stable certificate and key ownership across restart, with a visible rotation
  operation that does not train clients to ignore changed certificates; (3) configurable interface and port binding,
  automatic startup off by default, advertisement policy, connection and byte limits and useful failure status;
  (4) atomic snapshot handoff and recovery after interrupted writes, so only complete selected revisions are visible and
  editing a source file is not publishing; (5) a retention and garbage-collection policy including what Stop does and
  whether retained bytes remain for restart, with removal separate; (6) for governed content, current hosting
  permission after restart and on every authority change, refused after revocation or expiry; (7) acceptance with an
  independent ordinary client after Knot exits and after a Djinn restart, plus bind-conflict, malformed-request,
  resource-bound and shutdown tests.
- Needs: the Djinn entry gate question below
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:556-607,609-651`

### P2. Desktop publish control and main-broker owner attachment
The initial authoring caller is the CLI (`djinn-site publish`, `knot-site-publish` for the governed route). A Knot
desktop publish-to-Djinn control, and attaching the owner to the main broker for the governed route, are separate consumer
gates. Knot's existing in-process serving remains a separate editor-owned lifetime.
- Done when: the source sets none; the headed handoff is received separately (P4).
- Needs: P1 item 1
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:639-642`; `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:351-357,694-696`

### P3. Certificate rotation, installation, public binding
Explicit identity rotation, installation, public binding and additional protocols are further slices of the ordinary
service. Interface and port binding are configurable but the first implementation is loopback only.
- Done when: the source sets none beyond the contract's items 2 and 3 (P1) and the rule that public exposure has its own
  reviewed deployment receipt.
- Needs: P1 items 2 and 3
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:592-595,604-607,639-642`

### P4. Acceptance beyond native localhost
Every landed host was qualified through native localhost processes. Still separate gates, each with its own receipt:
a headed Knot window handoff; clean operating-system shutdown; public and two-machine exposure; power-loss; visual
Lagrange layout (stock-reader byte retrieval is not a visual or theme result); independent-machine delivery; abrupt
disconnect and actor-cancellation execution; Python and Go clients, browser and radio, and cross-machine acceptance of
governed Micron; oversized-request and flood acceptance for Micron fixtures; fetch-actor dispatch and transport
cancellation execution; and headed UI evidence for the browser receiving path, named separately from the native
retained-controller evidence (Reload, Stop with stale completion, Resource content and empty-page handling need
product controls).
- Done when: each gate named above has its own recorded receipt, none inferred from a neighbour.
- Needs: P1
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:644-651,686-692`; `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:72-73,124-133,162-163,216-217,231-232`

### P5. Governed selective hosting, peer acquisition and lineage
A device retains and serves publications accepted from a moot under current hosting authority: Gemot owns membership,
contribution and hosting permission, domain stores own durable records, transport owns replication, and Djinn
supervises and rechecks. Still open: production revision and hosting records, historical authority, Persona and device
binding, cross-Moot signed publication lineage, peer acquisition, hosting promises, network serving, actor wiring and
lifecycle, host revalidation and revocation, and protected delivery. Select accepted revisions and files by hash, retain
them under the relevant lease, and recheck hosting permission on restart and on authority change; discovery advertises
availability separately and retaining bytes grants neither editing rights nor decryption keys.
- Done when: not set by the sources; they fix the shape: selection by hash under a lease, permission rechecked on restart
  and on every authority change with refusal after revocation or expiry, a saved ordinary site still served with no moot,
  and acceptance on two machines.
- Needs: P1 item 6; Gemot's production records; `2026-10-28_custody_plan.md` for the worker's confinement
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:551-569,694-702`; `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:25-27,296-298,316-318,679-680,694-696`

### P6. Saved-site revision slices
Later slices: a history browser and restore workflow, a draft keystroke log, forks, federation and serving actors. A
collection policy for saved-site revisions is deferred: nothing is pruned implicitly, and a failed journal write may
leave an unreferenced blob.
- Done when: the source sets none; each slice names its own.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:251-252,259-260`

### P7. Wider Errand and smolweb coverage
Track the rest of the family by capability: a wire crate, a fetch route, a document engine and a durable resident each
establish different coverage. Open: further native serving (Spartan request bodies, Nex, Guppy transport behaviour);
writes and conversations (Titan upload, Spartan submission, Misfin mail,
DICT sessions), each with explicit authorization and its own host lifecycle, since read-publication selection grants no
submission or mailbox authority; and an inventory of DICT, FSP, Kepler, Scorpion, Text Protocol and Misfin APIs before
choosing fetch, serving, transfer or session integration. DICT and FSP need held session or transfer consumers; Kepler
needs native cache facts and a separately qualified TLS posture; Scorpion R and M need binary and range preservation
while S and I are incomplete; Text has no Errand route; Misfin has mailbox, storage and identity APIs and Comms read
integration but no product send or server composition. These are separate owner slices, not additions to the snapshot.
Micron is a document format with a Murm and Reticulum path, not an Errand scheme; Finger's WebFinger is HTTP and does
not widen Errand's non-HTTP router.
- Done when: for each capability added, exact native bytes and metadata, request effects, current authority or refusal
  and resident restart or cancellation behaviour are named and qualified, with snapshot V1 preserved.
- Needs: the pre-adapter comparison in Constraints
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:407-444,493-498`

### P8. Turnstone adoption
Turnstone adoption of the explicit-Submit effects rule stays separate: a Turnstone cohort pinned to an earlier Mere does
not gain it until its owner adopts and qualifies the supplier. Upstream HTTP referrer and security changes arrive with
the broader SDK selection, so unchanged browser policy is not claimed.
- Done when: a Turnstone cohort selecting the current Mere refuses Titan and Spartan submission targets before contact on
  initial and redirected reads, as the shared fetch supplier does.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:12,168-170,508-511`

### P9. Host hardening residue
Open behaviours of the landed hosts, none qualified: a record and content import that spans owners can leave accepted
facts when later retention refuses, and such an incomplete delivery refuses serving until explicitly repaired; the
native synchronous backend reads inline before Errand's ready future returns, so local I/O and JSON work is bounded by
content limits but not preempted by network timeouts; Spartan and Misfin accept loops spawn tasks whose join a
listener shutdown does not prove; the generic Gopher adapter drops Plus request facts and Finger's drops verbosity.
- Done when: the source sets none; each behaviour is either qualified with a regression or recorded as an accepted limit.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:359-364,470-473`

### P10. Portable reader verification (2026-10-29)

**Status:** implemented and locally checked, 2026-10-29. The first proof uses
complete evidence files and isolated reader processes; discovery, remote acquisition, custody acknowledgements and
physical-device failover remain follow-ons.

C's independently configured trust anchor is the Moot founder public key plus
Moot/collection identity (`CollectionRef`). A provider cannot replace that anchor.
C also supplies its evaluation time, retention posture and byte/record limits.
B supplies a complete public Gemot native drop (constitution, membership,
delegation and object/collection history as required by Gemot's export), plus
canonical snapshot V1 bytes. Neither side needs the author's private key to
verify or re-serve this evidence. Exporting a whole public drop may disclose
other public records in that Moot; selective evidence export is deferred.

The API is `mere_site::publication::reader::verify_reader_evidence`, behind the
existing `publication` feature. `ReaderTrust` carries the founder and collection;
`ReaderLimits` bounds history bytes and native-drop records. The provider evidence
is two byte slices: native drop and canonical V1 snapshot. `VerifiedPublication`
returns the opened site, evaluation time and `EvidenceScope::SuppliedHistoryOnly`.

The reader imports into a fresh, disposable in-memory Moot under the trusted
founder, uses Gemot's authorized collection fold to resolve exactly one effective
site contribution, and checks that its manifest names the supplied canonical
snapshot bytes. It retains the verified content only in disposable local storage
and opens through the existing publication adapter. No membership is authored,
no signing key is supplied, and no provider-selected release or clock is trusted.
Incomplete, invalid, ambiguous, withdrawn, expired or mismatched evidence refuses
when the existing authority path cannot establish an effective selection.

A successful result explicitly means **valid under supplied history**. It cannot
prove that a provider supplied all newer authority/revocation records. Each call
starts empty and has no remembered frontier, so it does not provide known-rollback
protection; P11 adds a separate persistent reader history owner. The snapshot V1
schema version is never a release sequence. Provider discovery and TLS identity
remain separate from this publication trust anchor.

- Done when: a cold child process verifies evidence re-exported by B after A and
  B's live objects are dropped; altered bytes, wrong anchors/collections, missing
  authority, invalid signed records, ambiguous selections, known withdrawal and
  bounded-input failures refuse; an older valid evidence set succeeds with its
  explicit freshness limitation. The proof covers Gemini, Scroll and Micron.
- Continuation: P11 implements remembered reader observations and rollback refusal.
  Follow-ons: minimal public authority export; transport framing/acquisition; daemon evidence delivery and
  exact-release custody acknowledgement; independent-machine serving.

### P11. Remembered reader authority (2026-10-30)

**Status:** implemented and locally checked, 2026-10-30. Extend P10 with a
caller-owned transactional Muniment store partitioned by the full
founder/Moot/collection trust anchor.

The API is `mere_site::publication::reader::history::ReaderHistory`. Its
`observe` method commits admitted history without requiring content, while
`verify` observes first and opens a snapshot against the merged history. A
successful read returns `EvidenceScope::SuppliedAndRememberedHistory`.

The reader reimports remembered Gemot evidence and the provider's complete drop
into disposable stores, then persists their re-exported union. It does not rank
releases by timestamp, schema version or a provider counter. A missing newer
operation in the incoming drop cannot remove an observed replacement, withdrawal,
revocation or conflict from the local authority fold.

Authority observation commits independently of content verification: valid new
history remains remembered even if the requested snapshot is missing, invalid or
no longer selected. Invalid imports and failed storage commits produce no new
observation. State metadata and full evidence bytes commit atomically with a
comparison against the previously read pair; concurrent writers retry rather
than overwrite a newer observation. Both reads use transactional snapshots.
Persist the reader's evaluation-time high-water mark and refuse a backwards
clock; this is local policy, not a publication sequence.

Success still cannot establish global newest state. Remembered history survives
ordinary process restart but relies on the host preserving the local store:
deletion or replacement of the entire store with an older valid backup is outside
this proof. Corrupt or incomplete state refuses instead of resetting silently.
Full retained history remains bounded; exceeding capacity refuses without pruning.

- Done when: a fresh process reopens reader storage and refuses an older snapshot
  after replacement; observation of withdrawal/revocation persists even when the
  associated page read refuses; malformed evidence, corrupt storage, failed writes,
  capacity, backwards time and concurrent stale commits cannot weaken memory.
- Follow-ons: host adoption, protected local storage/backup policy, selective
  evidence retention/checkpoint handling, acquisition/discovery and device proof.

## Open questions

- Does the `djinn 0.0.2` entry gate bind this service? The ordinary Gemini service shipped on 2026-09-13 without passing
  it; the cut places it here, and the question is carried in `2026-10-28_family_update_resident_plan.md`. From:
  `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:89-93`
- Which of the seven settle-and-verify items does the first slice already meet (loopback only, default-stopped startup
  with explicit `resume_on_restart`, a durable per-profile TLS identity, upload then explicit commit of a digest-verified
  snapshot, separate Stop and Remove)? The sources tick none off. From: `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:588-604,609-637`
- A progress line says revision logs and new persistence, Errand and Armillary adapters "remain subsequent slices", while
  later sections record the revision slice and the Gemini, Scroll and Micron runtimes as landed. Is anything from that
  sentence still open? From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:642-643`
- The wider-coverage table lists "Scroll metadata, language and abstracts" as further native serving, written on
  2026-10-21; the governed Scroll resident later preserved full and abstract bodies, language, UDC class, author and
  both dates. Does any Scroll serving remain open beyond multilingual selection? From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:382-385,418`
- Multilingual selection and fallback is a named design checkpoint: one page per path stays, and request language
  preferences introduce no variant negotiation. Is it to be designed, and by whom? From: `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:383-385,468-469`

## Findings

- 2026-10-30: `crates/domain/site/src/publication/reader/history.rs` retains a
  bounded full-evidence union plus trust identity, digest and evaluation-time
  high-water mark in one Muniment transaction. It imports into disposable Gemot
  stores, commits admitted authority before content verification, and compares
  both stored records before writing. Final read validation refuses a concurrent
  observation. `crates/domain/site/tests/reader/remembered.rs` covers Redb-backed
  process reopen, remembered replacement/withdrawal/revocation/conflict, invalid
  evidence and content, clock rollback, corrupt/incomplete state, transaction
  refusal/failure, capacity and stale writers. Re-export uses Gemot's existing
  complete native drop; history is not replaced by a release counter. Host
  adoption, pruning/checkpoint policy and protection against whole-store rollback
  remain open.

- 2026-10-29: `crates/domain/site/src/publication/reader.rs` verifies provider
  evidence using an independently configured founder and collection, Gemot native
  drop admission, shared collection selection and exact canonical snapshot bytes.
  Imports are disposable and never author reader membership. Success is limited
  to supplied history; withholding newer records and remembered rollback remain
  outside this stateless API. `crates/domain/site/tests/reader.rs` exercises cold
  child readers, B re-export after A drops, all three native profiles, replacement,
  withdrawal, membership revocation, expiry, wrong anchors, modified signed records
  under recomputed carrier integrity, missing authority, noncanonical/mismatched
  snapshots and caller limits. Current default limits are 8 MiB of history,
  4096 records, 1 MiB per record/manifest and the existing snapshot V1 byte cap;
  acquisition must enforce its own pre-buffer limits. Whole-drop evidence can
  include other public Moot records. This is file-delivered verification, not
  discovery, network delivery, durable custody or physical-device failover.

- 2026-10-21: `muniment` at the existing Eidetic pin already gives transactional directory and memory backends, immutable
  blobs and serializable journals; no Eidetic core change was needed. Eidetic's `manifest:` and `blob:` keys do not fit
  the directory backend's portable Windows filename rules, so disk qualification uses redb. Source:
  `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:269-275,313-315`.
- 2026-10-21: `gemot::moot` already supplies signed stable-identity contributions, capability-gated causal collections,
  forks and withdrawal; a collection may hold several sites, and the host selects a share explicitly at an observed
  version instead of inferring one from timestamps or titles. Source: same file `:302-312`.
- 2026-10-21: Errand's Titan fetch emits a zero-byte upload and its Spartan query fetch emits a request body; Mere's read
  redirect loop (`crates/system/fetch/src/lib.rs`) can cross into either, and Turnstone's interception of an initial
  Titan address does not establish redirect admission. A shared source does not establish native completeness. Source:
  same file `:470-480`.
- 2026-10-22: a NomadNet empty byte response cannot distinguish an empty page from a host refusal. Retinue's request
  parser keeps its historical empty-string and trailing-byte tolerance; the host adds no second MessagePack grammar.
  Source: same file `:118-120,204-206`.
- 2026-09-13: stock Lagrange refused the literal `127.0.0.1` URL with a certificate-verification error while the
  `localhost` hostname worked with verification on; use the hostname in acceptance runs. Source:
  `archive_docs/2026-10-28/2026-08-22_djinn_family_resident_services_plan.md:686-689`.
- 2026-10-22: serving a valid V1 date longer than Errand's 4096-byte response-line cap preserves the host and model
  contract but does not qualify the current Errand or browser client for that content. Source:
  `archive_docs/2026-10-28/2026-10-20_shared_site_model_plan.md:401-405`.

## Progress

- 2026-10-28: cut from the shared site model plan and the Djinn plan's section 13; nothing new landed.

- 2026-10-29: P10's bounded portable-reader API and isolated-process proof are
  implemented in the local worktree. The supplier proof preserves existing
  publication/history behavior and schema V1. Persistent rollback observations,
  selective evidence export, daemon delivery and consumer adoption remain open.

- 2026-10-30: P11's remembered-authority adapter and persistent reader-process
  proof are implemented locally. P10 remains available as a stateless verifier;
  both scopes still allow unknown newer history elsewhere.
