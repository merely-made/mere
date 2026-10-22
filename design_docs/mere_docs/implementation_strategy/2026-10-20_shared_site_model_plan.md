# Shared site model extraction

**Status (2026-10-22):** extraction and optional saved-site revision storage
landed. The optional Moot publication binding is landed, with Gemini,
Scroll and Micron as the first profiles. Snapshot V1 stays unchanged.
The Gemini, Scroll and Micron runtime continuations are implemented and qualified
through native processes.
Native Scroll replies now live in Errand and Knot uses them with unchanged V1
fixtures. Both shared read loaders require explicit Submit for Titan and Spartan
submission URLs; focused native tests qualify that rule.

## Scope and ownership

`mere-site`, at `crates/domain/site`, owns the portable native-site profiles,
page metadata, manifest validation, and `PublishedSnapshotV1` encoding and digest.
The default build has no filesystem, listener, runtime, editor, identity or
storage dependency. Optional `history` uses Eidetic's Muniment backend seam.
Optional `publication` composes Eidetic typed retention and Gemot's existing
signed contributions and collection authority.
Knot keeps file access, drafts, templates, checked saves and protocol adapters.
Djinn keeps custody, host identity, certificates, listener policy and resident lifecycle.

Cross-Moot publication lineage and peer acquisition are subsequent work.
The Gemini resident uses existing Errand and Armillary seams. Existing V1 limits and profile semantics remain
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

## Browser receiving (2026-10-22)

**Status:** shared receiving and native browser qualification are landed.
Client limits remain a separate lane.
Turnstone already has native NomadNet receiving through Retinue and explicit
Micron sessions; this continuation adopts the shared read-admission changes,
preserves response MIME through document construction and replacement, and
qualifies the existing product path against governed Djinn hosting.

Mere owns the MIME-preserving `EngineInput` lowering and session adapters.
Caller-supplied content must never trigger a fetch. Body-less loads retain the
fetcher's MIME, and streamed prefixes use explicit interpretation metadata.
Legacy same-address replacement retains known MIME; changing addresses clears
it. Native Scroll grammar must remain distinct from Gemtext, including a
level-five heading and a semantic Citation link. Text/plain stays literal.
Existing fields suffice; this does not add a generic protocol-metadata bag or
claim Scroll author/date/UDC retention or trust projection.

Turnstone owns source acquisition, request correlation, route/controller setup,
retained browser interactions and its coherent SDK receiving cohort. Native
NomadNet stays a destination and opaque path, with absent MIME; an empty byte
response cannot distinguish an empty page from a host refusal. Client caps,
deadline settings/defaults and cap ordering stay unchanged. The complete Retinue
crate at existing `bd8fe1e5` and qualified `5b1c5cac` has the same Git tree,
`d80e420d75809798401207655021812763d8664c`.

Done when focused shared tests cover supplied-body, response-MIME and replacement
paths; product tests retain the native Scroll construct and explicit Submit
admission; and a real Knot publisher/governed Djinn/Turnstone process proof
fetches native Micron into the ordinary browser controller, retains exact saved
bytes, frames the page and follows its qualified link. Reload, Stop/stale
completion, Resource content and honest empty-page handling need product controls.
Native retained-controller evidence and headed UI evidence must be named
separately. The native fixture calls the ordinary acquisition helper and hands
its completion through the product pipeline; Stop verifies late-result rejection.
Fetch-actor dispatch and transport cancellation execution need separate evidence.
Snapshot V1 stays unchanged. Whole-task receipts are capped at 100 MB
across repositories. Reuse stable Mere, Turnstone and Knot targets. The temporary
`Code/worktrees/mere-browser-receiving` checkout isolates the primary Mere's
foreign staged work and is removed after integration and publication.

Shared receiving source `af5c3ba85` passes 63 document unit tests, the streaming
integration test and 41 fetch controls, including the native Scroll grammar,
response MIME and explicit-Submit boundaries. Receipts live under
`support/receipts/smolweb-browser-receiving/`. Turnstone adopts that exact supplier
with its coherent published dependency cohort before product qualification.

Turnstone source `cab743c11` passes 69 focused product checks. The live
Knot `3596dce3` fixture passes against the governed Djinn host `1f660dcb4`:
the ordinary native acquisition helper receives the exact 57-byte index and
131,072-byte Resource, passes request correlation and the product reducer,
frames the registered Micron controller, and follows its qualified same-node
link. Existing V1, reused-link authority transitions and restart proofs pass
in the same fixture. Receipt:
`knot-editor/support/receipts/smolweb-browser-receiving/process-graceful.json`.

The first live attempt exposed premature client teardown: dropping an Endpoint
aborted its writer before queued Resource proof and LINKCLOSE packets drained,
holding remote admission until the existing request deadline. Ordinary page
fetches now use Retinue's existing graceful shutdown inside the unchanged outer
deadline, matching the submission path. The fixture handoff uses its existing
selected shutdown grace. The local receiver regression now requires sender-side
completion and joins every response task. First-phase host shutdown joins all
three sessions without cancellation. The original failed attempt is preserved.
Abrupt disconnect/actor-cancellation execution and headed interaction remain
separate gates.

The isolated older Turnstone checkpoint also required AccessKit type alignment and bounded API adoption;
59 of the product checks cover those affected areas, separately from ten
receiving/Micron checks. Micron and existing shared smolweb caps and deadlines
are unchanged. Existing upstream HTTP referrer/security changes arrive with the
broader SDK selection; this is not a claim that every browser policy is unchanged.
Existing buffered smolweb allocation limitations remain in the client-limits lane.

Final publication is based on current Turnstone `8eebb5a3`, preserving its
reading, navigation and exhaustive command metadata. Receiving source
`a622c9607` changes only the four receiving files and 62 canonical Mere revision
entries in the lockfile. Its manifest, package versions, dependency tables,
registry entries and other supplier pins are unchanged. The older checkpoint's
compatibility repairs and HTTP-policy uptake are historical evidence; they are
not reapplied to this current-code port.

The integrated build and 70 focused product checks pass, including existing
reading-state and toolbar navigation controls. The live process fixture passes
again with exact browser executable `4ef1b302…61757`, unchanged governed host
`d2ee5884…5055`, exact index/Resource bytes and all three first-phase sessions
joined without cancellation. Final receipt:
`knot-editor/support/receipts/smolweb-browser-receiving/process-integrated.json`.
Current SDK/build/test records live under
`turnstone/docs/receipts/native_browser_receiving_20261022/`.

## Governed Micron resident (2026-10-22)

**Status:** implemented and native process qualified on integrated source
`1f660dcb4`. Browser receiving and client limits keep their own gates.
Micron remains a saved native profile; NomadNet over Reticulum
is its transport. Retinue owns the native link and Resource exchange. Errand's
Gemini/Scroll projection and TLS identity do not participate in this adapter.

Djinn owns an optional `moot-site-nomadnet` backend, explicit native listener
policy and a separate persistent host identity. `configure_nomadnet` binds a
Micron publication to that identity. Every request reopens current collection
authority and retained content, including on a reused link. Native page paths
are the hashes of `/page` plus the canonical saved path. Nil and empty binary
request data read static pages; maps and nonempty data refuse with an empty byte
response. Missing or unavailable publications use the same reusable-link refusal.
Transport errors close the session without sending a second response. Retinue's
existing request parser also retains its historical empty-string and trailing-byte
tolerance; this host does not introduce a second MessagePack grammar. Configured
Resource limits apply to incoming transfers; outgoing pages retain the shared
snapshot page-size bound.

Done when native process tests preserve exact V1 bytes, large Resource content,
stable destination on restart, signed replacement, membership removal and
restoration, withdrawal and idle-session shutdown. Bound TCP interfaces, inbound
links, requests, Resources and session lifetime through host policy. Armillary
joins application sessions; endpoint shutdown and dedicated runtime destruction
finish transport work before the OS worker joins. Existing Gemini/Scroll process
fixtures must still pass. Python/Go, browser, radio and cross-machine acceptance
remain independent. Retain at most 100 MB of compact receipts across repositories,
reuse stable Cargo targets, and remove temporary integration isolation afterward.

Nine native host controls, eight existing resident controls, eight backend-disabled
controls and the disabled CLI check pass on the integrated source. The unchanged
Gemini/Scroll fixtures and new Micron fixture
all pass against executable SHA-256
`d2ee5884c925d91118efaee712237ab00d78d2c38b02d0684ec6b89f347c5055`.
Micron uses two actual Knot publisher processes and three resident processes,
checks exact 128 KiB Resource bytes and V1 identity, unavailable author paths,
stable destination/public key, and replacement, membership removal/restoration
and withdrawal on one link. Its admitted idle session is cancelled and joined;
the listener closes before acknowledgment/process exit. Authority refuses after
restart following withdrawal. Native interface-cap recovery and both request
and session deadlines have focused runtime coverage; oversized-request and flood
acceptance are not claimed by these fixtures.

The receiving merge preserves the separately qualified Knot checkpoint cohort
at `98d5212b`; its site library is unchanged. All other supplier pins stay fixed.
Initial and combined process proofs are retained separately in
[Knot's receiving receipt](https://github.com/merely-made/knot-editor/blob/main/support/receipts/micron-publication-runtime/integrated-process.json).
Strict Clippy remains blocked by 20 pre-existing Djinn diagnostics outside the three
changed host files. The [native receipt](../../../support/receipts/micron-publication-runtime/native.json)
records exact source, lock, executable, gate results and limits. The temporary
integration checkout is removed after publication; stable Mere/Knot targets are
reused and no isolated Cargo home or target is created.

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

## Gemini publication runtime (2026-10-21)

**Status:** landed and native qualified. Host authority and native
snapshot V1 stay unchanged.

1. Add a versioned endpoint binding with explicit pinned-contribution and
   follow-single-site-collection modes. Re-resolve current collection authority
   on each request; empty/ambiguous collections, withdrawal, missing public
   content and lost authority refuse. No timestamp chooses a revision.
2. Add Knot's saved-site publication command over host-supplied Moot/identity
   and Eidetic retention. Preserve drafts and report any contribution already
   authored if a later collection update refuses. Replacement uses existing
   collection operations and may transiently refuse while membership changes.
3. Extend Djinn's existing Gemini listener with Errand's Source adapter and
   Armillary ResidentTasks. Persist the endpoint binding and certificate,
   honor explicit restart policy, and join connection work on shutdown.
4. Done when an independent resident process reopens the binding and retained
   owners with Knot absent, serves exact saved bytes through real Gemini/TLS,
   preserves its certificate, reflects replacement/withdrawal, and refuses
   ambiguous/private/stale/foreign state. Preserve the original snapshot and
   resident tests. Retain compact receipts under 100 MB total, using stable
   Mere and Knot targets. No new worktree without a concrete collision.

### Runtime findings (2026-10-21)

Knot already supplies saved-file capture and native protocol replies. Djinn's
listener already owns loopback/TLS/certificate and connection ceilings. Errand
has a generic Source plus a Gemini Handler adapter; the native Scroll metadata
gap need not block Gemini. Armillary's current pin already supplies scoped
task admission, cancellation, joining and shutdown reports.

The first Knot product caller is `knot-site-publish`, with an existing Moot and
an explicitly provisioned identity on stdin. Its saved snapshot and signed drop
are imported explicitly into separate resident stores; no editor folder or author
key reaches the resident. Desktop controls and main-broker owner attachment are
separate consumer gates. The process receipt lives with the Knot command at
`knot-editor/crates/knot-site/tests/moot_site_runtime.rs` and is opt-in through
`runtime-acceptance` plus `DJINN_MOOT_SITE_EXE`.

The native synchronous backend reads complete inline before Errand's Send ready
future is returned. Local I/O/JSON work is bounded by content limits but not
preempted by network timeouts. Publication is revalidated per request; already
admitted responses may finish. Record/content import spans owners and may leave
accepted facts when later content retention refuses. Such incomplete delivery
refuses serving until explicitly repaired.

## Governed Scroll resident (2026-10-22)

**Status:** landed and native qualified on integrated `42b683108`.
The [native receipt](../../../support/receipts/scroll-publication-runtime/native.json)
links the exact build, focused host tests and independent product process gates.

- Extend the existing explicit `djinn-moot-site` caller with a persisted transport
  choice. Omitted transport keeps existing Gemini descriptors and commands
  compatible. The selected transport must match the snapshot profile on configure,
  start and every request; selecting a different profile cannot retarget a live
  listener. Explicit authorized content import remains separate from configuration.
- Reuse Djinn's loopback TLS identity, bounded connection admission and Armillary
  cancellation/join scope. Errand's native Scroll connection exchange owns request
  parsing and reply encoding; the host validates the requested loopback origin,
  rejects static-site query/credential/parameter targets, and reopens current
  publication authority before resolving a page.
- Preserve native full/abstract body bytes, declared language, UDC class, author
  and both dates. The existing single page per path remains the selection model;
  request language preferences do not introduce variant negotiation or fallback.
  Long dates accepted by snapshot V1 remain encodable within existing site bounds.
- Done when actual Knot publisher and independent resident processes serve exact
  native Scroll/TLS bytes with author files absent, retain transport/certificate
  across reopen, reflect replacement/current-authority loss and restoration,
  refuse withdrawal on restart, and cancel/join an admitted incomplete request
  before shutdown acknowledgment. Preserve the existing Gemini native gate and
  original resident/snapshot compatibility tests.
- Runtime, independent native process tests and read-only review occupy separate
  file lanes. The coordinator owns supplier repins, locks, integration and docs.
  Retain compact receipts below 100 MB across repositories; reuse stable Mere and
  Knot targets. Existing staged receipt collisions may require a temporary
  integration worktree under `Code/worktrees`, removed after publication.

Browser adoption, MIME/certificate/trust projection, main-broker attachment,
peer acquisition, hosting promises, protected delivery and signed cross-Moot
publication lineage retain their existing owners and acceptance gates.
The metadata-boundary receipt reads raw trusted TLS: Errand's current client
still caps each response line at 4096 bytes. Serving a valid V1 date beyond
that cap preserves the host/model contract but does not qualify the current
Errand or browser client for that content. Caller-configurable client bounds
are a separate receiving continuation.

## Wider Errand and smolweb coverage (2026-10-21)

The three initial publication profiles and the Gemini resident are starting
points. Track the rest of the family by capability; a wire crate, a fetch route,
a document engine and a durable resident each establish different coverage.
Errand's continuing repository is locally checked out as `smolweb`.

| Surface | Current source boundary | Publication continuation |
| --- | --- | --- |
| Errand scheme routing | Gemini, Gopher, Finger, Spartan, Nex, Guppy, Titan and Scroll in `errand::Scheme` | Inspect existing clients and protocol handlers before adding adapters; a fetch route does not establish resident serving. |
| Existing `errand::serve` adapters | Gemini, Gopher and Finger over `Source`; structural listings have native projections | Reuse these seams where faithful. Native documents retain their source format; a listing projection does not imply document conversion. |
| Further native serving | Scroll metadata/language/abstracts, Spartan request bodies, Nex and Guppy transport behavior | Preserve each native request/response contract and qualify it independently. Micron is a document format with a Murm/Reticulum transport path, not an Errand URL scheme. |
| Writes and conversations | Titan upload, Spartan submission and Misfin mail have effects; DICT has sessions | Give writes their own explicit authorization and host lifecycle. Read-publication selection alone grants no submission or mailbox authority. |
| Other family crates | DICT, FSP, Kepler, Scorpion, Text Protocol and Misfin exist independently of the scheme router | Inventory their actual APIs and consumer needs before choosing fetch, serving, transfer or session integration. Preserve cache, range, negotiation and interaction semantics where applicable. |

`SiteFormat` currently names Scroll, Gemini, Spartan and Micron. It is not a
catalog of every transport. Keep content profiles, transport selection and
endpoint policy distinct; extend the shared model only for demonstrated native
content requirements, with compatibility fixtures. Eidetic retention and Gemot
authority remain optional owners regardless of transport. Errand owns wire
behavior and protocol projection; Djinn owns endpoint custody/policy and
Armillary owns scoped execution. The native source adapter's local synchronous
I/O constraint remains visible when considering other host backends.

Rendering, browser routing, input/certificate conversations and trust projection
continue through the existing
[smolweb fidelity plan](../../nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md)
and `turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`.
Nematic lowers native documents; browser registration or rendering does not
qualify the publication host. Finger's separate WebFinger support is HTTP and
does not expand Errand's non-HTTP fetch router.

Before the next adapter slice, compare the consumer with the protocol crate's
existing request/response and server APIs. Its done-condition names exact
native bytes and metadata, request effects, current authority/refusal, and
resident restart/cancellation behavior as applicable. Preserve snapshot V1 and
qualify each capability actually added. This section records scope, not new
protocol implementation or acceptance evidence.

### Lane orchestration (2026-10-21)

Three bounded read-only lanes inspected serving, effects/sessions and browser
integration before implementation. Source basis: Mere `0606dfaad`, Errand's
local `smolweb` checkout `50702b440` and Turnstone `9c8420d5` (locked Mere
`500bc78a`). The Errand crate sources match the current `dce1f085` receiving pin.
Research retained source references only; the entire orchestration retains at
most 100 MB of compact receipts. Stable repository targets are reused.

| Lane | Owner and bounded files | Next done-condition |
| --- | --- | --- |
| Native Scroll supplier, landed | Errand `scroll-protocol` wire encoder, optional connection exchange and focused tests; coordinator owns supplier plan/index and source publication | Exact native headers/body, ordered request languages/abstract flag, bounds, header-injection refusal, caller timeout and cancellation; no listener or spawned tasks. Source `3d2ce3a5`. Canonical supplier plan: `smolweb/design_docs/implementation_strategy/2026-10-21_native_scroll_server_plan.md`. |
| Existing Knot receiver, landed | A test lane owns only `crates/knot-site/tests/scroll_wire.rs`; coordinator owns encoder adoption, supplier pins/locks and receiving docs | Unchanged literal full/abstract fixtures and real local TLS tests pass before/after extraction, including saved replacement, refusal and stop. Snapshot V1, host policy and renderer suppliers preserved. Source `02c5a2d`; 22 standalone cases, root check and strict Clippy pass. |
| Effect admission, qualified | Mere fetch actor and existing submission controls; worker owns the fetch crate source/tests, coordinator owns plan/source integration | Initial/redirected Titan and Spartan query reads refuse before contact. Exact explicit submission bytes, identity presentation and trace redaction pass in the 40-case suite plus one isolated trace child; strict Clippy passes. Existing `SmolwebSubmission` remains the write path. |
| Document read admission, qualified | A separate worker owns only `document-lanes/src/remote.rs` and its existing smolweb test module | The direct Errand read loader refuses submission targets before trust setup or transport. Five focused cases pass, including zero contact, exact queryless Spartan bytes and unchanged `None`/redirect behavior. MIME, session and accessibility work stays with its existing owner. |
| Browser receiving, existing owner queue | Mere fidelity S1/WS2 and Turnstone unusual-protocols SC5/S8/SC6; no files acquired by this publication lane | A Scroll-only construct must reach the native lowerer through retained MIME. Preserve the existing accessibility/source/build queue; certificate-gated Scroll and trust projection have separate gates. |

Findings that constrain the next slices:

- At the research baseline, Scroll's wire crate had native request/header
  parsing and stream-generic client exchange but no native server seam. The
  shared encoder and connection exchange are now landed; Knot retains its
  blocking TLS host. Its one-page-per-path language behavior stays unchanged by
  extraction; multilingual selection/fallback remains a named design checkpoint.
- Gopher's generic adapter drops Plus request facts; Finger's drops verbosity.
  A shared source does not establish native completeness. Existing Spartan and
  Misfin accept loops spawn tasks whose join is not proven by listener shutdown;
  host qualification must account for admitted connection work explicitly.
- Errand's Titan fetch emits a zero-byte upload, Spartan query fetch emits a
  request body, and Mere's read redirect loop can cross into either path
  (`crates/system/fetch/src/lib.rs`). Turnstone intercepts an initial Titan
  address, which does not establish redirect admission. The policy checkpoint
  was resolved directly by Mark: **Require explicit Submit**. Apply the rule
  before every initial or redirected read exchange, preserving the separate
  typed submission path and protocol wire semantics.
- A second review found `document-lanes::RemoteFetcher` calling Errand directly
  for body-less document loads. Its admission guard follows the same rule;
  browser/image routing beyond that confirmed call graph is not inferred.
  Fetch trace addresses also need userinfo and Titan path parameters removed,
  in addition to query/fragment redaction. Qualify actual rejected-read trace
  events while preserving the real submission target and credentials.
- `document-lanes` session spawning drops `SessionSpawnRequest.content_type`;
  its smolweb engine builds `EngineInput` without MIME, while Scroll lowering
  uses MIME to distinguish scrolltext. A heading/link-only browser test cannot
  establish this fidelity. Turnstone's identity-origin code permits Gemini and
  Titan, not Scroll; Nematic trust still defaults to `Unknown` despite durable
  TOFU at the browser host. These remain browser-owner findings.
- DICT and FSP need held session/transfer consumers; Kepler needs native cache
  facts and a separately qualified TLS posture; Scorpion R/M needs binary/range
  preservation while S/I remain incomplete; Text has no Errand route yet.
  Misfin already has mailbox/storage/identity APIs, with Comms read integration
  but no established product send/server composition. These are separate future
  owner slices rather than additions to `PublishedSnapshotV1`.

The Scroll consumer review found that valid UTC timestamps can retain more than
1024 fractional digits. The encoder's line cap now derives from validated
metadata within the existing snapshot bound; a new arbitrary date-length limit
would reject existing V1 content. Regression coverage includes both long dates
and a 1024-byte Unicode author. Supplier and consumer receipts live with their
owners at `smolweb/support/receipts/native-scroll-server/supplier.json` and
`knot-editor/support/receipts/native-scroll-server/consumer.json`.

The implemented effects rule is qualified on the shared fetch supplier; a
Turnstone cohort still pinned to an earlier Mere does not gain that behavior
until its browser owner adopts and qualifies the supplier. This lane leaves the
existing browser source/accessibility/build queue intact.

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

- 2026-10-22: governed Scroll source `e3ddbedcd` is integrated with current
  upstream `27f7aaf2e`, retaining its complete SDK/renderer supplier cohort.
  The clean receiving resident builds locked/offline; five publication-host
  controls and eight original resident controls pass. Both native product
  fixtures pass against that exact executable, with the original Gemini test
  unchanged. Scroll uses two publisher and three resident processes, preserves
  literal full/abstract bytes and long metadata, reopens the same certificate,
  follows signed replacement/current membership, refuses withdrawal on restart,
  and reports one cancelled admitted request with zero task panics/rejections.
  [Receiving receipt](../../../support/receipts/scroll-publication-runtime/native.json).
  Browser adoption/client bounds, Micron hosting and main-broker attachment
  retain their separate gates. Stable repository targets are reused; the
  temporary integration checkout is removed after publishing the receipts.

- 2026-10-22: the three research lanes led to two bounded implementation slices.
  Errand `3d2ce3a5` adds native Scroll replies and a caller-owned connection
  exchange; Knot `02c5a2d` adopts the encoder with the exact donor wire fixtures
  and snapshot V1 retained. Mere's fetch actor and direct document loader now
  apply Mark's explicit-Submit ruling before transport, including redirect hops
  in the shared fetch loop. The fetch suite passes 40 cases plus one isolated
  trace child and strict Clippy; the document-loader gate passes five cases.
  [Lane qualification](../../../support/receipts/smolweb-lanes/qualification.json)
  records hashes, commands, ownership limits and supplier/consumer evidence.
  Turnstone adoption, retained Scroll MIME, certificate/trust projection and a
  governed Scroll resident remain separate receiving and host gates.

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

- 2026-10-21: Gemini runtime source `0a00e4c2e`, shared binding supplier
  `a845b0b22`, and Knot publisher `b37be4f` are integrated through Mere
  `b775eae24`. The final locked resident binary is built at `527136589`;
  resident sources, manifests and lock are identical on the receiving head.
  The native process gate passes with two actual Knot publisher processes,
  three resident processes and independent retained owners. It checks exact
  full Gemini/TLS wire bytes, certificate continuity, replacement, current
  membership removal/restoration, withdrawal and refusal on restart. A stalled
  TLS-admitted request is cancelled and joined before shutdown acknowledgment.
  The existing eight resident cases, shared 16 cases plus doctest, Knot's 18
  site cases, ordinary root check and strict shared/Knot Clippy pass.
  [Native receipt](../../../support/receipts/gemini-publication-runtime/native.json).
  The CLI is the first product caller. Desktop controls/main broker attachment,
  Scroll/Micron runtime serving, peer acquisition, hosting-promise integration,
  protected delivery and signed cross-Moot publication lineage remain open.
  Isolation was required by the concurrent root-lock collision; stable Mere and
  Knot targets were reused and no new isolated Cargo home was created.
