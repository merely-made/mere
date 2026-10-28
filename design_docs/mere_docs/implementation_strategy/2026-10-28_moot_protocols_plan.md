# Moot Protocols Plan

**Status:** 2026-10-28: cut from `2026-05-05_protocol_architecture_plan.md`, `2026-06-26_federation_interop_plan.md` and `2026-06-30_commitment_proof_interface_plan.md`, archived in `archive_docs/2026-10-28/`. The protocol architecture plan is largely overtaken (the transport is p2panda, verso is folded into `inker::flip`, the vault lock plan rules the tiers consent and not custody); what holds is its iroh and iroh-blobs rows and the vault types, and what stays open is discovery and proof (old Phase 2D), the protocol mods and primitive moot nodes (Phase 3) and self-host generalisation (Phase 4), all unbuilt. Federation interop (schema lenses, capability-scoped sharing) is scoped and not started. The shared commitment type interface is landed in `crates/system/proofs`; the proof types, verifiers and backends are not.

**For:** how Mere composes protocols that are not its native peer exchange: the multi-protocol identity vault and the discovery and proof of handles; Matrix, Nostr, IRC and ATproto as protocol mods and primitive moot nodes a moot can host for visitors who have no mod installed; and the two federation mechanisms painful to retrofit once peers exchange data, schema lenses and capability-scoped sharing; plus the typed commitments and proofs that moot epochs, role rosters and receipts need. Opened by the 2026-05-05 protocol architecture critique (discovery is not proof; Cable stays in Murm), the 2026-06-26 federation scoping and the 2026-06-30 decision to expose typed commitments and not a `MerkleTree`.
Not here: the p2panda transport, replication, the Moot constitution, roster and drops (Murm runtime plan); the vault lock, unlock-tier custody, the threat statement and the Secret Service (custody plan); persona egress and unlinkability (browser carrier plan); the Dramatis tier architecture's gazette announcing (Dramatis, rulings 3 and 6); Misfin's server side (nematic and the smolweb plans); calls (calls and media plan).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- What holds of the old transport sections: iroh stays the QUIC byte plane under p2panda-net and iroh-blobs is used directly by `p2panda_transport.rs`; the iroh-gossip and iroh-docs rows, the Cable choice rule and the ALPN ledger are superseded by the p2panda transport. iroh-docs is a swappable projection, with p2panda-sync and willow-rs as fallbacks. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:3-14, 104-118, 151-157, 890-894`
- The vault types hold (`repos/dramatis/crates/personae/src/vault.rs`): slots are Direct (bytes the vault stores) or Bootstrap (a bootstrap secret plus an encrypted state directory the SDK owns); recovery is lineage-dependent (locally derived, locally generated and externally registered, externally issued, externally rooted) and the vault UI says per slot what losing the device means; profiles are independent with Element-style concurrent sub-instances and lazy-load by default; `IdentityVault<S>` implements `IdentityProvider` for the current profile; the vault lands before any protocol mod that needs non-Mere credentials. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:159-175, 293-333, 393-398`
- Unlock tiers are consent, not custody (vault lock ruling 12); the vault assumes isolation level 0 (every mod trusted inside the process) and its threat model belongs to the vault threat statement. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:3-12, 334-392`
- Discovery is not proof. WebFinger is unauthenticated discovery; bidirectional verification is the default, and a one-sided entry is an unverified hint tagged as such on any identity card. NIP-05 is its own surface (`/.well-known/nostr.json`), not a WebFinger rel; Matrix and ATproto keep their own resolution flows with WebFinger auxiliary. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:401-430, 473-516`
- Doc signing: Mode 1 (self-host) uses TLS only; Mode 2 (third-party host) and Mode 1.5 (kith and kin bridge) use TLS and a JWS-wrapped doc signed by the master key alone; SXG is out of scope. Mode 2 is a convenience with an explicit "your handle is fragile" warning; self-hosting is the recommended path. Publication is one toggle per identity slot, with a granular config editor for advanced users. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:517-554`
- The Murm and Moothold axis is durable group with membership versus ad-hoc peer interaction. Cable stays in Murm ("the soul of it is bilateral"); Matrix and Nostr are fully Moothold including DMs; Misfin's client and exchange side stays in Murm (relaxed 2026-06-01) while nematic hosts the server side; Cable owns durable membership events and gossip owns live status. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:563-589, 775-793`
- Pattern A pins a protocol client in the user's orrery and needs the mod; Pattern B pins it inside a moot, the moot host carries the session, and visitors need no mod. Phase 3 ships puppet mode only (the moot's bot owns room membership and visitor messages are prefix-attributed) because federation propagation of a visitor's real identity cannot be undone; portal mode is later research; Pattern B starts as a single moot host relay; every protocol mod reuses the `mere/moothold/primitive-node/v1` ALPN. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:590-666, 785`
- Mods are sequenced by cost tier: T1 (wrap a maintained SDK: IRC, Nostr, Matrix, then ATproto) before T2, and T3 (implement from spec: MLS, Briar) each get their own plan. IRC is the first concrete mod, Nostr second (its NIP-39 back-claim is the beachhead for proof), Matrix third. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:695-722`
- Commitments: expose typed commitments and proofs, never a `MerkleTree`; `DigestAlg` (the digest family), `CommitmentScheme` (how the digest reads as a proof root) and blob content digests are three meanings that must not collide; the first build implements only `MerkleV1`, `SparseMerkleV1` and `MmrV1` and the other schemes stay enum slots until scale or privacy pressure justifies them. From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:23-85, 138-152`
- p2panda boundary for proofs: never replace p2panda operation hashes with commitment digests, never treat a Merkle or MMR root as the p2panda log head, never put scheme-specific proof logic in operation header extensions, never use application proofs as a substitute for p2panda signature or backlink validation. From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:154-208`
- Three senses of "capability" must not be conflated: the in-app permission spine (the capability-gate catalogue), DocumentScript's WASM confinement, and the federation sense (what a remote peer may read or write). Federation grants are read-only first and namespace-scoped first (finer per-node only if a need proves it); a layered sketch (token format, delegation, revocation, how each rides p2panda) precedes any borrow; revocation without a server rests on an epoch or key-rotation model. From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:73-106`
- Meadowcap is a possible structural-capability provider at Gemot's injected authorization seam, not a linked Murm dependency and not a replacement for reputation, group membership or keys. From: `archive_docs/2026-10-28/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md:1273-1277`

## Phases

### V. Vault storage backends and unlock methods
`IdentityVault<S>`, `Profile`, the slot types, `InMemoryStorage` and `PassphraseEncryptedStorage` (Argon2id and ChaCha20-Poly1305, atomic-rename single file) are built. Left from old Phase 2C: `OsKeychainStorage` (the intended default desktop backend), `HardwareTokenStorage` and `HostBridgeStorage` (browser extension host), the passkey and hardware-token unlock methods (unbuilt), and the per-slot recovery UX that the identity-vault sub-plan owns.
- Done when: the backends in the storage table implement `IdentityStorage` against a vault opened on exactly one, tier and method are user-overridable per slot, and the vault UI shows per slot what losing the device means.
- Needs: the custody plan, which holds the lock, the unlock-method build (first build carries passphrase and Windows Hello; Linux passphrase alone) and the Linux root in the OS keyring.
- Rulings: vault lock ruling 12 (consent, not custody).
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:293-333, 368-369, 727-736`

### D1. Discovery endpoints and publication (old Phase 2D)
A host serves `/.well-known/webfinger` and `/.well-known/nostr.json` from the current profile's slots, with JWS-signed documents for Mode 2 and Mode 1.5 (`sign_discovery_doc` on the vault, master key only), per-identity publication policy keyed to the slot, third-party static publication (Mode 2), a kith and kin bridge over `mere/webfinger-bridge/v1` (gossip the doc id, fetch by content address, Mode 1.5), and the `mere:previous-handle` chain (`previous_handles: Vec<HandleClaim>`) so migration is verifiable without a live old host. Multi-host fan-out and resolver-fallback ordering are a polish track. The old host for Mode 1, `verso-tile`'s HTTP listener, no longer exists.
- Done when: a listener serves `/.well-known/webfinger` and `/.well-known/nostr.json` populated from the current profile's slots (Mode 1, on whatever host the open question settles); a publish-static-doc flow exists for both endpoints (Mode 2); the bridge relays a signed cached doc when the primary host is unreachable (Mode 1.5); a previous-handles chain verifies continuity across a handle move.
- Needs: V; a host for Mode 1 (see the open question on whether Mode 1 survives the tier rulings); iroh-blobs and gossip on the p2panda transport for the bridge.
- Rulings: discovery is not proof; doc signing; per-identity publication.
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:430-460, 517-559, 738-748, 799`

### D2. Back-claim collection and verification
When a slot is added, offer to publish the matching back-claim (NIP-39 kind:0 metadata for Nostr, Nostr first since NIP-39 is mature; a signed room post for Matrix; ATproto's bidirectional handle and DID flow); the vault tracks back-claim status per slot so identity cards render verified or unverified.
- Done when: a Nostr slot publishes its back-claim, and a resolver surfaces the binding as verified only when both sides claim each other.
- Needs: P2's Nostr mod.
- Rulings: discovery is not proof.
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:415-425, 506-516, 746-748`

### P1. Primitive moot node runtime and the IRC mod
`PrimitiveMootProtocol`, the `PrimitiveNodeRelay` machinery and the `mere/moothold/primitive-node/v1` handler for a single moot host; the IRC mod (T1, wrapping the `irc` crate, IRCv3 capabilities scoped) as the first concrete consumer, Pattern A pin and Pattern B puppet-mode relay, integrated with the vault `Irc` slot. IRC is a good stress test because it needs a long-lived TCP connection, exactly the workload that benefits from the moot hosting on the visitor's behalf.
- Done when: an IRC node pins in a user's orrery (Pattern A) and a moot-hosted IRC node renders for a visitor with no mod installed through the moot's relay (Pattern B, puppet mode).
- Needs: the p2panda transport (the old iroh transport rows are superseded); V for the `Irc` slot.
- Rulings: puppet mode only; single-moot-host relay; the content-shape question below.
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:590-672, 750-757, 881-888`

### P2. Nostr, Matrix and ATproto mods
Nostr (T1): Pattern A pin; Pattern B with moot-bot signing (puppet) or visitor-`nsec` signing per moot configuration; drives D2. Matrix (T1, the first Bootstrap-category consumer): wraps `matrix-rust-sdk`, SDK state in a vault-provided encrypted directory, puppet mode only. ATproto (T1 to T2): wraps `atrium-rs`, same Bootstrap pattern, with spec churn expected. The mooting protocol adapters for Matrix and Nostr land as those backends land.
- Done when: each mod passes Pattern A and Pattern B (puppet) against its vault slot (`Nostr`, `MatrixBootstrap`, `AtprotoBootstrap`).
- Needs: P1; V.
- Rulings: cost tiers; puppet mode.
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:695-722, 754-763`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:987-988`

### P3. Moot membership roster as shared mutable state
The first concrete consumer of a CRDT-shaped store was ruled to be the moot membership roster (concurrent additions and removals by several admins), then shared bookmark and pin lists; co-op session state stays on Cable (durable) or gossip (live). Not started on iroh-docs, which the 2026-05-31 caution treats as a swappable projection. The Murm runtime plan carries a roster lane on the shared muniment store and processor.
- Done when: a roster of who belongs to a moot is a CRDT with concurrent additions and removals, on whichever substrate the open question below settles.
- Needs: the Murm runtime plan's Moot domain service (M3).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:759-760, 787, 890-894`

### P4. Isolation upgrade and the scripting host probe
Capability tokens (isolation level 1): the vault checks a per-mod token, scoped to declared protocol ids, before serving a slot; triggered by the Matrix mod, the first mod touching Bootstrap credentials end to end. Level 2 (Wasm sandbox) follows the scripting probe; level 3 (per-mod processes with IPC) is a long-term goal. The scripting host probe evaluates Boa, Extism and Wasmtime against the capability surface a concrete scripted-mod need requires (network, vault, filesystem, rendering by impression channel only); until a need appears, scripting is future scope.
- Done when: the Matrix mod lands with vault-side per-mod token enforcement; the probe runs against a concrete scripted-mod use case and names the capability surface.
- Needs: P2 for level 1; a concrete scripted-mod need for the probe.
- Rulings: isolation upgrade trigger is the Matrix mod.
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:371-392, 677-694, 757-761, 790`

### P5. Self-host generalisation (old Phase 4)
Misfin server self-host via nematic and a listener (the pattern applied to Misfin); Cable cabal 24/7 seeder mode (a reachable Mere seeds a community's cabal); a decentralised storage time bank as a design probe only, named so future work can slot in.
- Done when: the self-host-with-fallback pattern (Modes 1, 1.5 and 2) is applied to a Misfin server; a reachable Mere acts as a cabal seeder backed by blobs and gossip; the time-bank item is a research document, not an implementation.
- Needs: D1's HTTP listener (Misfin); the persistent Cable store and blob transport on the p2panda transport (seeder).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:456-472, 765-773`

### P6. Held protocol items
Each waits on its own trigger: the Phase 4 portal-mode bridge (a Matrix portal homeserver or per-visitor ActivityPub actors, when a community asks for E2EE or per-visitor reactions in Pattern B); the Pattern B scaling cliff (kith and kin volunteer relays, hosted fallback, a time bank, when the first moot hits it); the scripting host capability surface; distributed Mode 2 publication policy; the MLS sibling Bilateral protocol (T3, its own plan); T2 mods after T1; Tox, Hypercore and Briar as research probes; the generic chat-impression schema as a long-term unification target.
- Done when: each item's trigger arrives.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:456-458, 624, 675, 689-691, 709-715, 763, 785, 794-799`

### F1. Engram schema lenses
Cambria-style bidirectional lenses between schema versions, built from primitive ops (rename, hoist, wrap, add-with-default, remove, head), so a peer applies a lens chain to read an engram authored under a schema it does not hold. A registry keyed by `(from_schema, to_schema)` sits beside the schema vocabulary (`SchemaRef`, now in `crates/eidetic/eidetic-core/src/schema.rs`; engrams are codicils and no registry type exists), consulted at the boundary where content crosses peers and not in the hot graph path. Lens core, registry and JSON ops come first and are unit-testable without the network.
- Done when: the lens core, registry and ops pass engram-local tests; then a sync-boundary hook projects a received codicil to the local shape through a lens chain and a write flows back through the same lens.
- Needs: the open questions on authorship, storage and the failure mode.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:14-50, 108-113`

### F2. Capability-scoped subgraph sharing
A federation access-control primitive: peer X may read subgraph S, revocably, without a server. Scope is a namespace (a community-derived namespace maps to a capability scope), the grant dovetails with Standing (the `gemot/standing/v1` lane, code in `crates/moot/mien`) as the trust receipt, and a grant's reach graduates with trust. Servitor grants (`Mode` Read < Write < Delegate, `Grant` with expiry), Gemot's capability-scoped `MootAuthorizationRequest` and Commons group-key epochs now exist. The kith capability sharing tail is carried here: the mesh lease site `crates/mesh/mesh/src/lease.rs:22-26` owes the kith plan, which is re-scoped onto the shipped vocabulary (`servitor::cap`, Gemot `typed_authorization.rs`, Personae delegation certificates), keeping only the mesh-specific parts (claim validation in the board fold, epoch revocation, the six done-conditions); notochord may already answer "was this chain valid at T". The mere-native Meadowcap structural-cap layer and the signet trust-plane rung (a first non-mere consumer) are carried here too.
- Done when: the layered borrow sketch and token model are written; then read-only namespace grants work over p2panda; then revocation without a server; then write-capabilities as a later rung.
- Needs: the borrow decision; the open question on what is left of this section.
- Rulings: federation capability is a third sense; read-only and namespace-scoped first.
- From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:52-113`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:350-358, 379-382, 1189`

### Q1. Proof types, verifiers and scheme backends
`InclusionProof`, `NonInclusionProof`, `AppendProof` and `SetRelationProof` (with `SetRelation`) exist in no `.rs` file, nor do the callers' facts `verify_membership`, `verify_non_membership`, `verify_append` and `verify_overlap`. The first backends are `MerkleV1`, `SparseMerkleV1` and `MmrV1`; the default scheme map is role rosters sparse Merkle, receipt logs MMR, chunks Merkle, checkpoint logs MMR, mesh result batches Merkle or MMR depending on append pressure, and private membership overlap deferred to `PsiCardinalityV1`. Landed: `Digest`, `Commitment`, `CommitmentScheme` (with `DigestV1`), `CommitmentDomain` and `BlobRef` in `crates/system/proofs`.
- Done when: callers ask for facts through the `verify_*` functions and never walk a tree, over the three first backends; `AccumulatorV1`, `VectorCommitmentV1` and `PsiCardinalityV1` stay enum slots until scale or privacy pressure justifies them.
- Needs: -
- Rulings: p2panda boundary for proofs; typed commitments, not a tree.
- From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:87-152, 220-246`

### Q2. Moot epoch headers and role witnesses (done-conditions 1 and 2)
`MootEpochHeader` stores role commitments and not raw `*_root: Hash` fields; work receipts carry a typed `RoleWitness` and an optional `DelegationProof`. The illustrative `MootEvent` sketch (`EpochDeclared`, `WorkReceiptIssued`, `GrantIssued`, `GrantRevoked`) is not known to have landed. The retired bounty verification economy's design (`BountyEvent`, `ResultSpec`, `MootEpochHeader`, `WorkReceipt`, `RoleWitness` over `proofs::Commitment` and `proofs::Digest`; its standing ledger is now mien's Standing) is carried here, with nothing built.
- Done when: `MootEpochHeader` and the receipt types exist with typed commitments, and a verifier checks a `RoleWitness` against a role commitment.
- Needs: Q1.
- Rulings: p2panda boundary for proofs.
- From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:174-194, 210-227`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1100-1103`

### Q3. Checkpoint commitments and wire documentation (done-conditions 4 and 5)
Storage checkpoints distinguish chunk commitments from checkpoint-log commitments: retention checkpoints bind a `BlobRef` plus a `StorageCheckpoints` commitment while `StorageChunks` is unused, so this is a judgement call. The p2panda wire documents must say that commitments live in event bodies and projections and not operation headers; no such statement was found.
- Done when: the chunk-versus-log distinction is decided and reflected in the checkpoint types, and the wire documentation carries the commitments-in-bodies statement.
- Needs: -
- Rulings: p2panda boundary for proofs.
- From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:215-227`

## Open questions

- Do the protocol architecture plan's surviving rows (iroh, iroh-blobs) and its vault section stay authoritative in a plan of this scope, or are they history that defers to the p2panda transport for the transport and to the vault lock plan for the vault? From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:14`
- Does the old §4 survive the Dramatis tier architecture's rulings 3 and 6, which put announcing in gazette (castellan issues, gazette announces) as a Djinn service exporting static well-known files first, and the loss of `verso-tile` as Mode 1's host? Drop Mode 1 from this plan, or keep §4 as the long-range Mode 1 beside the tier rulings. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:3-12, 434-445`
- Do the unbuilt storage backends and the passkey and hardware-token unlock methods belong to this plan or to the custody plan, which holds the vault lock, the Linux OS-keyring root and the resident's custody tiers? From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:297-303, 368-369`
- Pattern B content shape: server-rendered HTML streamed to the visitor's tile, wire-protocol bytes with a moot-supplied Wasm viewer (the current bias, since it composes with the scripting host), or a generic Mere chat-impression schema (a long-term unification target)? From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:667-676`
- Which substrate carries the moot membership roster CRDT, iroh-docs (not started) or the muniment and p2panda roster lane the Murm runtime plan carries? From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:759-760, 890-894`
- Lens authorship: hand-written per schema bump, derived from a schema diff, or both? Lens storage: their own codicil kind that federates like other content, or a side registry? When no lens chain resolves, render the content inert (the received-content rule, proposed) or fail? From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:42-47`
- What is left of capability-scoped sharing now that servitor, Gemot and Commons provide grants, scoped authorization and group-key epochs: narrow it to the remaining borrow choice (Meadowcap, UCAN or Keyhive) over those primitives, or rule it superseded by them? From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:58-71, 99-103`
- The trust-plane umbrella name; the grant format (Meadowcap versus Biscuit, with the kith re-scope); and a first non-mere consumer as the one open rung. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:379-382`
- What does done-condition 3 (kith grants naming a delegation commitment without assuming a Merkle tree) target now that the kith capability sharing plan is retired: drop it, re-target it to the capability model's servitor grants, or hold it until a grant surface names a delegation commitment? From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:214, 229-232`
- Mesh result batches: `MerkleV1` or `MmrV1`, depending on append pressure (undecided). From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:146`

## Findings

- 2026-05-05: Matrix, ATproto and ActivityPub identity cannot be modelled as flat bytes (device keys, cross-signing keys, pickled Olm and Megolm sessions), so the vault holds a bootstrap secret and the SDK owns the rest; recovery means something different per lineage; the vault does not isolate mods inside one process, so at-rest encryption and per-session unlock defend against disk theft and not in-process exfiltration. From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:818-838`
- 2026-10-08: swap is not defended; a page written to the pagefile or hibernation file while the vault is unlocked survives the lock (vault threat statement, ruling 82). From: `archive_docs/2026-10-28/2026-05-05_protocol_architecture_plan.md:386-391`
- 2026-10-06: engrams are codicils (`c51b9704`, 2026-08-31) and "alembic" now names Distillery's recall component, so the lens registry's neighbour is eidetic-core's schema vocabulary; Tessera is Standing, with code in `crates/moot/mien`. From: `archive_docs/2026-10-28/2026-06-26_federation_interop_plan.md:34-41, 87-98`
- 2026-10-06: the landed proof types differ from the plan's sketches (`DigestV1` added; `StandingReceipts` where the sketch had `TesseraReceipts`, surviving as a serde alias after the 2026-08-31 terminology ruling); the code samples are illustrative, not compile-ready. From: `archive_docs/2026-10-28/2026-06-30_commitment_proof_interface_plan.md:70-85, 220-227`

## Progress

- 2026-10-28: cut from `2026-05-05_protocol_architecture_plan.md`, `2026-06-26_federation_interop_plan.md` and `2026-06-30_commitment_proof_interface_plan.md`; nothing new landed.
