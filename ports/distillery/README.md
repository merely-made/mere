# distillery

**Distillery** is the model-works port of the Mere platform. Its first authority
slice is implemented as the first real consumer of the mesh host supervisor,
which distillery has held as its `mesh_host` module since 2026-09-23 (it was
the `mere-mesh-host` crate).

A distillery takes a raw mash and runs it, batch by batch, through stills into
something concentrated. This port is that works for models: the harness where
a ring of devices runs inference, embedding, and eventually training jobs. It
splits in two, in the castellan mold: an embeddable half any host app composes
(the job board, lease and heartbeat tiles, the device roster, retention and
device-policy panes, model-manifest browsing, a minimal streaming console;
views that render what the supervisor reports, never a placebo), and an
authority half that lives with the device (the supervisor loop that owns mesh
host ticks, device conditions, the resource registry, and blob custody; it
answers to the owner, and reclaim always wins).

The boundaries are the point: not [esp](https://crates.io/crates/esp) (the
inference/embedding seam crate stays the portable burn boundary; distillery
drives it), not mere-mesh (job grammar and leases are substrate; distillery
embeds and renders them, and holds the supervisor as `mesh_host`), not servitor
(petition gating stays the gate's office), and not turnstone (the flagship
embeds the same views; distillery is the standalone works).

The trainer lane (Distillery-as-trainer, per the geist brief) lands here
later, behind its own plan: training is one more job the works runs.

## Current slice

`Distillery<B>` composes a `MeshHost<B>`, drives its non-blocking supervisor
ticks, and owns an explicit maintenance operation. Maintenance authors an
owner-governed retention checkpoint, asks the mesh store which blob references
are safe against both the checkpoint and the current replay tail, and can
release this mesh's named custody tags. Collection is off by default.

Releasing a tag is a logical custody change. Physical bytes disappear only
when the collecting `mere-transport` store finds no remaining tag from another
mesh or subsystem. A content hash reused by an unfinished or post-checkpoint
job remains protected.

D1 adds `ResidentAuthority<B>`, the reusable body of a long-lived device
process. The owner supplies supervisor, maintenance, and physical collection
cadences; the type deliberately has no default policy. `ResidentStorage` opens
a disk-backed collecting blob store and binds its custody tags to one mesh. The
resident owns that storage, the live p2p transport, the mesh host, and their
shutdown order. Shutdown waits for the LogSync drain, every topic manager, and
the top-level actor task to drop their store handles, so a successful return is
a real restart boundary rather than an asynchronous stop notification.

Every supervisor turn, completed maintenance run, unchanged maintenance turn,
maintenance refusal, and shutdown request is an ordered `ResidentReceipt`.
Scheduled maintenance keeps running after a visible refusal such as a live
lease. A supervisor failure is fatal. An unchanged event frontier is reported
without authoring another checkpoint.

The D1 proof reopens both the redb mesh store and collecting blob store after a
real resident run. The job remains terminal and released content stays
released across that restart boundary.

`ResidentAuthority::enable_record_ingress(config)` now opts into separately
Notochord-admitted exact inventory and original signed-record submission on that
same endpoint and mesh store. Both actions share two active calls, four pending
sessions and an arrival-to-result-FIN deadline. The owner supplies authority,
clock and availability lifetime; `MeshStore::accept_outcome` remains the domain
admission path. Volatile retained originals expire or FIFO-evict independently
of durable operations. The [local composition and receipt](../../crates/mesh/mesh/examples/inventory_ingress/README.md)
cover actual RaptorQ recovery, QUIC handoff, preserved signed bytes, action
separation, shared-budget refusal and fresh-process durable reopening.

With `inference-model`, `InferenceLane::try_open_with_journal` adds a
synchronous checkpoint barrier to the actor. The decoder records exact session,
rendered prompt, model-token IDs and random-draw position. Text becomes visible
after the commit; resumed generation rebuilds KV state by prefill. The retained
record uses Eidetic's `InferenceRunRecord` schema, keeping paused, failed,
cancelled partial and finished states distinct. I4 adds retained passage inputs.

Layered library rows use the same CPU/GPU lane entry points. The lane retains
its existing library owner and borrows it briefly for each content-verified
layer artifact. It releases that lock before tensor work. GPU layer reads and
execution stay inside the foreground guard, preserving trainer reclaim and
release on pause. The streamed loader and weight index form part of the exact
session identity. I6's larger-than-memory measurement remains a separate gate.

`PersonaRunJournal` stores signed, encrypted per-device logs through muniment
and joins the caller's existing p2panda endpoint. Token entries contain only new
IDs and text. A fork is reported rather than silently choosing a continuation.
A host can bind a journal to its existing encryption group through
`RunGroupProtection`. Djinn uses the personal graph's live keyring and membership,
with separately rooted, domain-attested writers on its existing endpoint.
Current epochs require a current group member before insertion. Retained
epochs admit historical writers named by the retained key agreement, and
local historical reads survive removal of a writer. Unlock and key revisions
reopen reconciliation against the retained frontier; a child whose parent has
not arrived is retried without persisting a dangling checkpoint. The
persona-only constructor remains the local journal and test seam.

`InferenceLane::configure_recall` borrows a `RecallCorpus` and explicit
`RecallConfig`. `PageTextCorpus` reads already-owned Eidetic page-text storage,
verifying immutable blob bytes; other source owners supply the same snapshot
and current-version contract with capture, clip or Fleece identities. The
BM25 `DocumentIndex` is keyed by passage content references. Its deterministic
Unicode chunks retain the source version, canonical text hash and half-open
Unicode scalar selector. Reconfigure recall to rebuild a changed corpus.

`distillery.inference/ask-recall-v1` takes an `ask` and optional source `scope`.
It renders selected passage handles through the declared context template and
retains the exact supplied passages as sealed native Eidetic artifacts. Stale
or unavailable sources are excluded. An ask without usable passages is retained
as failed before generation, with structured stale-source details. A stub
cannot answer retained material. Current answer views distinguish validated
model markers from host annotations of complete verbatim passage matches;
partial matches and invented handles do not become citations. `min_quote_chars`
defaults to 32 non-padding characters; zero disables host quote annotations. Both citation
bases carry exact answer spans and require the source owner's current version.
Historical input evidence remains readable when a source changes.

Context-bearing checkpoint entries carry an authenticated `required_semantics`
marker: older checkpoint readers reject them instead of silently losing context.
Ordinary ask serialization remains compatible. Lock clears the derived recall
index with the run materialization; the composing source owner keeps its own
storage and disclosure authority.

`inference-wire` exposes the run table, controls and recall DTOs without ESP or
a decoder. An admitted source owner can publish its retained text through
`distillery.inference/publish-corpus-v1`, then ask with
`distillery.inference/ask-published-v1`. Each publication names an application
session namespace and exact text hashes. Only its live publisher can replace
it; disconnect or vault lock withdraws the publication. The host controls recall
limits. Inline JSON is capped at 128 KiB and oversized publications are refused.
Published material stays in memory; supplied passages and run records use the
existing sealed journal. Published checkpoints require `recall/published-v1`.

See the [resident inference plan](../../design_docs/mere_docs/implementation_strategy/2026-10-15_resident_inference_and_recall_plan.md).

`probe/` is D2's standalone headed-browser evidence surface. Its first MiniLM
row localized and recovered two Burn/CubeCL browser defects. The configured
D2c embedding matrix now passes BGE Micro, MiniLM, E5-small, and E5-base in
headed Chromium, including integrity reopen, numerical references, worker
termination, and frame receipts. The largest success is a 437,955,512-byte F32
artifact. A first 269,060,552-byte BF16 SmolLM2 decoder row also passes exact
Transformers and ESP NdArray token references, cold/warm worker streaming,
cooperative token-boundary cancellation, explicit `GPUDevice.destroy()`, and
exact recovery in a fresh worker. The upper model boundaries and physical GPU
allocation release remain outside the browser API's observable surface and are
not actionable browser gates. The separate native Windows driver receipt proves
the two supported plain-WGPU remote cycles release their model allocations with
zero retained growth; larger browser rows remain consumer-gated.

The lease-bound Burn Remote adapter is now live behind Distillery's `remote`
feature. It mounts on the resident p2panda/Iroh endpoint, admits only a signed
claim projected from the host's exact active mesh lease, and closes every
matching session before owner reclaim is authored. The native MiniLM forcing
fixture under `probe/remote-fixture` exercises ESP over that adapter on WGPU;
the checked-in receipt records the numerical, cancellation, and fresh-session
recovery boundaries.

The installed-authority slice now persists one explicit Personae profile at
`<data-root>/distillery/settings.json`. The record contains no seed, vault
secret, scheduler cadence, device policy, or mesh-retention policy. Distillery
opens no vault (dramatis repo plan, D5): `InstalledAuthority::open` takes the
profile's identity from djinn, either the unlocked vault (djinn's own works
lane) or the derived keys djinn releases for `InstalledAuthority::release_salts`
(D11). The mesh author is derived under `mesh::MESH_AUTHOR_SALT` and is also
the transport identity (vault lock ruling 92). Per-mesh private paths live
below `<data-root>/distillery/meshes/<mesh-id>/`.

`distillery-installed configure --data-root <path> --profile <id>` persists
that selection. `inspect` asks djinn's custody route for the roster and the
protection, reports the identity as pending when djinn is absent or Locked
(D12), and intentionally does not start a resident. The binary is built from
djinn's package since DR-C. A real start
still requires a mesh owner to supply its store and retention policy, and a
device owner to supply `HostConfig` and `ResidentSettings`; Distillery must not
invent scheduler or device-policy authority. The read-only
`distillery.installed.v1` Cambium surface projects this same profile/path model
plus caller-supplied resident settings and receipts. Turnstone registration,
an operational product composition, model manifest browsing, the streaming
console, and the first deterministic trainer remain later slices.

The installed library path passes five exact-source tests covering explicit
settings replacement, stable profile reopen and private paths, caller-owned
resident binding, surface snapshot rendering, and exact resident receipt
rendering. The configure/inspect binary check and package Clippy with warnings
denied also pass against the exact-source graph. The full workspace and
Turnstone composition gates remain separate.

Lives in the [mere](https://github.com/merely-made/mere) workspace at
`ports/distillery`.

## License

MPL-2.0
